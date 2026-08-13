//! Fill-in probes for local runtimes. Never owns the `/model` catalog.

use std::num::NonZeroU64;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use indexmap::IndexMap;

use super::super::config::{Config, LOCAL_DEFAULT_CONTEXT_WINDOW, ModelEntry};

const PROBE_TIMEOUT: Duration = Duration::from_millis(350);
const PROBE_CACHE_TTL: Duration = Duration::from_secs(5);

/// Well-known local OpenAI-compat ports. Probe only; never retarget a model.
pub(crate) const KNOWN_RUNTIME_PROBES: &[(&str, u16, &str)] = &[
    ("ollama", 11434, "http://127.0.0.1:11434/v1"),
    ("llama.cpp", 8080, "http://127.0.0.1:8080/v1"),
    ("lmstudio", 1234, "http://127.0.0.1:1234/v1"),
    ("openai-compat", 8000, "http://127.0.0.1:8000/v1"),
];

pub(crate) fn runtime_detect_disabled() -> bool {
    std::env::var("GROK_SKIP_RUNTIME_DETECT")
        .ok()
        .is_some_and(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
}

pub(crate) fn is_ollama_base_url(base_url: &str) -> bool {
    reqwest::Url::parse(base_url)
        .ok()
        .and_then(|u| u.port_or_known_default().map(|p| (u, p)))
        .is_some_and(|(u, port)| {
            port == 11434
                && u.host_str()
                    .is_some_and(|h| matches!(h, "127.0.0.1" | "localhost" | "::1"))
        })
}

/// `http://127.0.0.1:11434/v1` → `http://127.0.0.1:11434/api/show`
pub(crate) fn ollama_show_url(base_url: &str) -> Option<String> {
    let mut url = reqwest::Url::parse(base_url).ok()?;
    url.set_path("/api/show");
    url.set_query(None);
    Some(url.to_string())
}

pub(crate) fn parse_ollama_show_context(value: &serde_json::Value) -> Option<u64> {
    if let Some(info) = value.get("model_info").and_then(|v| v.as_object()) {
        for (key, val) in info {
            if key.ends_with("context_length")
                && let Some(n) = val
                    .as_u64()
                    .or_else(|| val.as_i64().and_then(|i| u64::try_from(i).ok()))
                && n > 0
            {
                return Some(n);
            }
        }
    }
    if let Some(params) = value.get("parameters").and_then(|v| v.as_str()) {
        for line in params.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("num_ctx") {
                let n = rest.trim().parse::<u64>().ok()?;
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Fill sentinel windows from a lookup. Skips keys that set `context_window` in config.
pub(crate) fn fill_unset_context_windows(
    cfg: &Config,
    catalog: &mut IndexMap<String, ModelEntry>,
    lookup: impl Fn(&str, &str) -> Option<u64>,
) {
    for (key, entry) in catalog.iter_mut() {
        if cfg
            .config_models
            .get(key)
            .and_then(|o| o.context_window)
            .is_some()
        {
            continue;
        }
        let current = entry.info.context_window.get();
        if current != LOCAL_DEFAULT_CONTEXT_WINDOW && current != 200_000 {
            continue;
        }
        if !is_ollama_base_url(&entry.info.base_url) {
            continue;
        }
        if let Some(n) = lookup(&entry.info.base_url, &entry.info.model)
            && let Some(cw) = NonZeroU64::new(n)
        {
            tracing::info!(
                model_key = %key,
                slug = %entry.info.model,
                from = current,
                to = n,
                "filled context_window from runtime (config field was unset)"
            );
            entry.info.context_window = cw;
        }
    }
}

pub(crate) fn ollama_show_context_window(base_url: &str, slug: &str) -> Option<u64> {
    if runtime_detect_disabled() {
        return None;
    }
    let show = ollama_show_url(base_url)?;
    let slug = slug.to_owned();
    // reqwest::blocking owns a tokio runtime; never drop it on a worker thread.
    std::thread::spawn(move || ollama_show_context_window_inner(&show, &slug))
        .join()
        .ok()
        .flatten()
}

fn ollama_show_context_window_inner(show: &str, slug: &str) -> Option<u64> {
    let client = reqwest::blocking::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .ok()?;
    let response = client
        .post(show)
        .json(&serde_json::json!({ "name": slug }))
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let value: serde_json::Value = response.json().ok()?;
    parse_ollama_show_context(&value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProbe {
    pub name: &'static str,
    pub base_url: &'static str,
    pub reachable: bool,
}

static PROBE_CACHE: Mutex<Option<(Instant, Vec<RuntimeProbe>)>> = Mutex::new(None);

/// Last completed probe (possibly empty). Does not block.
pub fn last_runtime_probes() -> Vec<RuntimeProbe> {
    PROBE_CACHE
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|(_, v)| v.clone()))
        .unwrap_or_default()
}

pub fn refresh_runtime_probe_cache() {
    if runtime_detect_disabled() {
        return;
    }
    if let Ok(g) = PROBE_CACHE.lock()
        && let Some((at, _)) = g.as_ref()
        && at.elapsed() < PROBE_CACHE_TTL
    {
        return;
    }
    let probes = probe_known_runtimes();
    if let Ok(mut g) = PROBE_CACHE.lock() {
        *g = Some((Instant::now(), probes));
    }
}

/// Human group title for a model `base_url`.
/// Port map + whatever is actually installed (PATH / app bundle).
pub fn runtime_group_label(base_url: &str) -> String {
    runtime_display_name(base_url, &detect_installed_runtimes())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstalledRuntime {
    Ollama,
    LlamaCpp,
    LmStudio,
    Omlx,
    Vllm,
}

pub fn detect_installed_runtimes() -> Vec<InstalledRuntime> {
    let mut found = Vec::new();
    if command_on_path("ollama") {
        found.push(InstalledRuntime::Ollama);
    }
    if command_on_path("llama-server") || command_on_path("llama-cli") {
        found.push(InstalledRuntime::LlamaCpp);
    }
    if command_on_path("lmstudio") || macos_app_exists("LM Studio.app") {
        found.push(InstalledRuntime::LmStudio);
    }
    if command_on_path("omlx")
        || command_on_path("mlx_lm")
        || command_on_path("mlx-lm")
        || command_on_path("mlx_lm.server")
    {
        found.push(InstalledRuntime::Omlx);
    }
    if command_on_path("vllm") {
        found.push(InstalledRuntime::Vllm);
    }
    found
}

fn command_on_path(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return true;
        }
    }
    false
}

fn macos_app_exists(bundle: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        std::path::Path::new("/Applications").join(bundle).is_dir()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bundle;
        false
    }
}

pub fn runtime_display_name(base_url: &str, installed: &[InstalledRuntime]) -> String {
    let origin = normalize_origin(base_url);
    let port = reqwest::Url::parse(&origin)
        .ok()
        .and_then(|u| u.port_or_known_default());
    match port {
        Some(11434) if installed.contains(&InstalledRuntime::Ollama) => "Ollama".into(),
        Some(11434) => "Ollama".into(),
        Some(8080) if installed.contains(&InstalledRuntime::LlamaCpp) => "llama.cpp".into(),
        Some(8080) => "llama.cpp".into(),
        Some(1234) if installed.contains(&InstalledRuntime::LmStudio) => "LM Studio".into(),
        Some(1234) => "LM Studio".into(),
        Some(8000) if installed.contains(&InstalledRuntime::Omlx) => "oMLX".into(),
        Some(8000) if installed.contains(&InstalledRuntime::Vllm) => "vLLM".into(),
        Some(8000) => "OpenAI-compat".into(),
        _ if origin.is_empty() || origin == "other" => "Other".into(),
        _ => origin,
    }
}

pub fn status_online(reachable: bool) -> &'static str {
    if reachable { "online" } else { "offline" }
}

pub fn normalize_origin(base_url: &str) -> String {
    let Ok(url) = reqwest::Url::parse(base_url) else {
        return base_url.trim_end_matches('/').to_string();
    };
    let host = url.host_str().unwrap_or("");
    match url.port_or_known_default() {
        Some(port) => format!("{}://{host}:{port}", url.scheme()),
        None => format!("{}://{host}", url.scheme()),
    }
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

/// Collapse loopback aliases so `localhost:11434` and `127.0.0.1:11434` group together.
/// Always uses `127.0.0.1` for loopback (stable, matches baked defaults).
pub fn canonical_runtime_origin(base_url: &str) -> String {
    let Ok(url) = reqwest::Url::parse(base_url) else {
        return normalize_origin(base_url);
    };
    let host = url.host_str().unwrap_or("");
    let host = if is_loopback_host(host) {
        "127.0.0.1"
    } else {
        host
    };
    match url.port_or_known_default() {
        Some(port) => format!("{}://{host}:{port}", url.scheme()),
        None => format!("{}://{host}", url.scheme()),
    }
}

/// `127.0.0.1:11434` for the header parenthetical.
pub fn runtime_endpoint_paren(base_url: &str) -> String {
    let origin = canonical_runtime_origin(base_url);
    reqwest::Url::parse(&origin)
        .ok()
        .map(|u| {
            let host = u.host_str().unwrap_or("");
            match u.port_or_known_default() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_string(),
            }
        })
        .filter(|s| !s.is_empty())
        .unwrap_or(origin)
}

/// Probe well-known loopback ports. Does not change catalog or URLs.
pub fn probe_known_runtimes() -> Vec<RuntimeProbe> {
    if runtime_detect_disabled() {
        return KNOWN_RUNTIME_PROBES
            .iter()
            .map(|(name, _, base_url)| RuntimeProbe {
                name,
                base_url,
                reachable: false,
            })
            .collect();
    }
    std::thread::spawn(probe_known_runtimes_inner)
        .join()
        .unwrap_or_default()
}

fn probe_known_runtimes_inner() -> Vec<RuntimeProbe> {
    let client = match reqwest::blocking::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    KNOWN_RUNTIME_PROBES
        .iter()
        .map(|(name, _, base_url)| {
            let url = format!("{base_url}/models");
            let reachable = client
                .get(&url)
                .send()
                .is_ok_and(|r| r.status().is_success());
            RuntimeProbe {
                name,
                base_url,
                reachable,
            }
        })
        .collect()
}

pub fn format_runtime_doctor_lines(probes: &[RuntimeProbe]) -> String {
    let mut out = String::from("Reachable local runtimes (probe only; config.toml owns /model):\n");
    for p in probes {
        let mark = status_online(p.reachable);
        let name = runtime_group_label(p.base_url);
        out.push_str(&format!("  - {name} {}  {mark}\n", p.base_url));
    }
    out
}

/// Live fill used at catalog resolve. No-op under `cfg(test)` so unit tests stay hermetic.
pub(crate) fn apply_live_context_window_fill(
    cfg: &Config,
    catalog: &mut IndexMap<String, ModelEntry>,
) {
    if cfg!(test) || runtime_detect_disabled() {
        return;
    }
    fill_unset_context_windows(cfg, catalog, ollama_show_context_window);
    refresh_runtime_probe_cache();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::{Config, resolve_model_list};

    #[test]
    fn parse_show_prefers_model_info_context_length() {
        let v = serde_json::json!({
            "model_info": { "qwen3.context_length": 262144 },
            "parameters": "num_ctx                        4096"
        });
        assert_eq!(parse_ollama_show_context(&v), Some(262144));
    }

    #[test]
    fn parse_show_falls_back_to_num_ctx() {
        let v = serde_json::json!({ "parameters": "stop \"<|end|>\"\nnum_ctx 32768\n" });
        assert_eq!(parse_ollama_show_context(&v), Some(32768));
    }

    #[test]
    fn fill_skips_explicit_config_window() {
        let raw: toml::Value = toml::from_str(
            r#"
            [model.m5]
            model = "qwen3.6:35b-a3b"
            base_url = "http://127.0.0.1:11434/v1"
            context_window = 200000
            "#,
        )
        .unwrap();
        let cfg = Config::new_from_toml_cfg(&raw).unwrap();
        let mut catalog = resolve_model_list(&cfg, None);
        fill_unset_context_windows(&cfg, &mut catalog, |_, _| Some(262144));
        assert_eq!(catalog["m5"].info.context_window.get(), 200_000);
    }

    #[test]
    fn fill_applies_when_window_unset() {
        let raw: toml::Value = toml::from_str(
            r#"
            [model.m5]
            model = "qwen3.6:35b-a3b"
            base_url = "http://127.0.0.1:11434/v1"
            "#,
        )
        .unwrap();
        let cfg = Config::new_from_toml_cfg(&raw).unwrap();
        let mut catalog = resolve_model_list(&cfg, None);
        fill_unset_context_windows(&cfg, &mut catalog, |url, slug| {
            assert!(url.contains("11434"));
            assert_eq!(slug, "qwen3.6:35b-a3b");
            Some(131072)
        });
        assert_eq!(catalog["m5"].info.context_window.get(), 131_072);
    }

    #[test]
    fn fill_ignores_non_ollama_urls() {
        let raw: toml::Value = toml::from_str(
            r#"
            [model.omlx]
            model = "mlx-community/x"
            base_url = "http://127.0.0.1:8000/v1"
            "#,
        )
        .unwrap();
        let cfg = Config::new_from_toml_cfg(&raw).unwrap();
        let mut catalog = resolve_model_list(&cfg, None);
        let before = catalog["omlx"].info.context_window.get();
        fill_unset_context_windows(&cfg, &mut catalog, |_, _| Some(99999));
        assert_eq!(catalog["omlx"].info.context_window.get(), before);
    }

    #[test]
    fn localhost_and_loopback_share_canonical_origin() {
        assert_eq!(
            canonical_runtime_origin("http://localhost:11434/v1"),
            canonical_runtime_origin("http://127.0.0.1:11434/v1")
        );
        assert_eq!(
            runtime_endpoint_paren("http://localhost:11434/v1"),
            "127.0.0.1:11434"
        );
    }

    #[test]
    fn display_name_uses_omlx_when_installed_on_port_8000() {
        assert_eq!(
            runtime_display_name("http://127.0.0.1:8000/v1", &[InstalledRuntime::Omlx]),
            "oMLX"
        );
        assert_eq!(
            runtime_display_name("http://127.0.0.1:8000/v1", &[]),
            "OpenAI-compat"
        );
        assert_eq!(
            runtime_display_name("http://127.0.0.1:11434/v1", &[InstalledRuntime::Ollama]),
            "Ollama"
        );
    }

    #[test]
    fn ollama_show_url_strips_v1() {
        assert_eq!(
            ollama_show_url("http://127.0.0.1:11434/v1").as_deref(),
            Some("http://127.0.0.1:11434/api/show")
        );
    }
}
