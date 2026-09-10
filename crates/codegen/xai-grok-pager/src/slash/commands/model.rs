//! `/model` (alias `/m`): switch the model and optionally its reasoning effort.
//! Chained autocomplete: after picking a reasoning-supported model, the trailing space re-opens the dropdown into a `low|medium|high|xhigh` sub-menu.

use agent_client_protocol as acp;
use xai_grok_shell::sampling::types::supports_reasoning_effort_meta;

use crate::acp::model_state::ModelState;
use crate::app::actions::Action;
use crate::slash::command::{
    AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand, slash_meta,
};
use crate::slash::commands::effort_levels::build_effort_arg_items;

/// Switch the active model (and optionally its reasoning effort).
pub struct ModelCommand;

impl SlashCommand for ModelCommand {
    slash_meta! {
        name: "model",
        aliases: ["m"],
        description: "Switch the active model",
        usage: "/model <name> [effort]",
        takes_args: true,
        args_required: true,
        session_scoped: true,
        // The dashboard offers `/model` to pick the model for the next spawned agent (intercepted in `dispatch_dashboard_dispatch_slash`).
        offered_when_session_less: true,
        arg_placeholder: "<model> [effort]",
    }

    fn suggest_args(&self, ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
        if ctx.models.is_empty() {
            return None;
        }

        // Effort phase if input is "<reasoning-model> ", else model phase.
        if let Some(model_id) = detect_effort_phase(ctx.models, args_query) {
            return Some(build_effort_items(ctx.models, &model_id));
        }
        if args_query.trim().is_empty() {
            Some(build_model_items_grouped(ctx.models))
        } else {
            Some(build_model_items_flat(ctx.models))
        }
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return CommandResult::Error("Usage: /model <name> [effort]".into());
        }

        // Prefer an exact full-string catalog match first. Model display names often contain spaces ("Grok 4.5").
        // If we split on the last token first, a shorter catalog entry ("Grok") would steal the prefix and treat "4.5" as an effort level
        if let Some(id) = ctx.models.resolve_by_name_or_id(trimmed) {
            return CommandResult::Action(Action::SetDefaultModel(id));
        }

        // A trailing effort token on a reasoning model makes a session-scoped switch (not persisted as default)
        // Resolve via the shared gate so a rejected level (e.g. `none` on grok-4.5) reports the effort error with the model's offered ids.
        // Without it the fall-through reports "Unknown model: … none"
        if let Some((prefix, token)) = split_trailing_token(trimmed)
            && let Some(id) = resolve_model(ctx.models, prefix)
            && ctx
                .models
                .available
                .get(&id)
                .map(supports_reasoning_effort)
                .unwrap_or(false)
        {
            return match ctx.models.resolve_effort_for_model(&id, token) {
                Ok(effort) => CommandResult::Action(Action::SwitchModel {
                    model_id: id,
                    effort: Some(effort),
                }),
                Err(err) => CommandResult::Error(err.message()),
            };
        }

        CommandResult::Error(format!("Unknown model: {trimmed}"))
    }
}

/// Look up a model by case-insensitive display name OR model id match.
fn resolve_model(models: &ModelState, name: &str) -> Option<acp::ModelId> {
    models.resolve_by_name_or_id(name)
}

fn supports_reasoning_effort(info: &acp::ModelInfo) -> bool {
    supports_reasoning_effort_meta(info.meta.as_ref())
}

/// Split `args` into `(prefix, last_token)` on the final whitespace run.
/// Returns `None` when there is no interior whitespace to split on.
/// The token is resolved to an effort against the picked model's options by the caller.
fn split_trailing_token(args: &str) -> Option<(&str, &str)> {
    let (prefix, last) = args.rsplit_once(char::is_whitespace)?;
    let prefix = prefix.trim_end();
    if prefix.is_empty() || last.is_empty() {
        return None;
    }
    Some((prefix, last))
}

/// Returns the matched model id when `args_query` is `"<reasoning-model> ..."`.
/// Candidates are tried longest name first to disambiguate names that share a prefix.
fn detect_effort_phase(models: &ModelState, args_query: &str) -> Option<acp::ModelId> {
    let mut candidates: Vec<(&acp::ModelId, &str)> = models
        .available
        .iter()
        .filter(|(_, info)| supports_reasoning_effort(info))
        .map(|(id, info)| (id, info.name.as_str()))
        .collect();
    candidates.sort_by_key(|(_, name)| std::cmp::Reverse(name.len()));

    for (id, name) in candidates {
        if args_query.len() > name.len()
            && args_query.is_char_boundary(name.len())
            && args_query[..name.len()].eq_ignore_ascii_case(name)
            && args_query[name.len()..].starts_with(char::is_whitespace)
        {
            return Some(id.clone());
        }
    }
    None
}

fn header_row(display: &str, description: &str) -> ArgItem {
    ArgItem::section(display, description)
}

fn model_base_url(info: &acp::ModelInfo) -> String {
    info.meta
        .as_ref()
        .and_then(|m| m.get("baseUrl"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn compact_context_window(tokens: u64) -> String {
    if tokens >= 1024 && tokens.is_multiple_of(1024) {
        format!("{}k", tokens / 1024)
    } else if tokens >= 1000 {
        format!("{}k", tokens / 1000)
    } else {
        tokens.to_string()
    }
}

fn model_context_tokens(info: &acp::ModelInfo) -> Option<u64> {
    info.meta
        .as_ref()
        .and_then(|m| m.get("totalContextTokens"))
        .and_then(|v| v.as_u64())
}

/// Catalog key · window, plus a short toml description when it is not the name.
fn model_secondary_text(id: &acp::ModelId, info: &acp::ModelInfo) -> String {
    const MAX_DESC: usize = 48;
    let key = id.0.as_ref();
    let window = model_context_tokens(info).map(compact_context_window);
    let mut parts = Vec::new();
    if info.name != key {
        parts.push(key.to_string());
    }
    if let Some(w) = window {
        parts.push(w);
    }
    let mut text = parts.join(" · ");
    if let Some(desc) = info.description.as_deref().map(str::trim) {
        if !desc.is_empty()
            && desc.len() <= MAX_DESC
            && !desc.contains('\n')
            && desc != info.name
            && desc != key
        {
            if !text.is_empty() {
                text.push_str(" · ");
            }
            text.push_str(desc);
        }
    }
    text
}

fn model_arg_item(
    id: &acp::ModelId,
    info: &acp::ModelInfo,
    current_id: Option<&acp::ModelId>,
) -> ArgItem {
    let is_current = current_id == Some(id);
    let supports = supports_reasoning_effort(info);
    let display = if is_current {
        format!("{} (current)", info.name)
    } else {
        info.name.clone()
    };
    let insert_text = if supports {
        format!("{} ", info.name)
    } else {
        info.name.clone()
    };
    ArgItem::row(
        display,
        info.name.clone(),
        insert_text,
        model_secondary_text(id, info),
    )
}

/// Flat list for type-to-find (no section headers).
fn build_model_items_flat(models: &ModelState) -> Vec<ArgItem> {
    let current_id = models.current.as_ref();
    models
        .available
        .iter()
        .map(|(id, info)| model_arg_item(id, info, current_id))
        .collect()
}

/// Catalog models grouped by runtime origin. Probe status is header-only.
fn build_model_items_grouped(models: &ModelState) -> Vec<ArgItem> {
    use indexmap::IndexMap;
    use xai_grok_shell::agent::models::{
        canonical_runtime_origin, last_runtime_probes, runtime_header_title, status_online,
    };

    let current_id = models.current.as_ref();
    xai_grok_shell::agent::models::refresh_runtime_probe_cache();
    let probes = last_runtime_probes();
    let mut groups: IndexMap<String, Vec<(&acp::ModelId, &acp::ModelInfo)>> = IndexMap::new();
    for (id, info) in &models.available {
        let origin = canonical_runtime_origin(&model_base_url(info));
        let key = if origin.is_empty() {
            "other".to_string()
        } else {
            origin
        };
        groups.entry(key).or_default().push((id, info));
    }

    let mut items = Vec::new();
    for (origin, members) in groups {
        let probe_status = probes
            .iter()
            .find(|p| canonical_runtime_origin(p.base_url) == origin)
            .map(|p| status_online(p.reachable));
        let title = runtime_header_title(&origin, probe_status);
        items.push(header_row(&title, ""));
        for (id, info) in members {
            items.push(model_arg_item(id, info, current_id));
        }
    }
    items
}

/// One row per effort level for the `/model` chained effort phase.
/// `insert_text` is `"ModelName high"` so selecting a row completes both tokens.
fn build_effort_items(models: &ModelState, model_id: &acp::ModelId) -> Vec<ArgItem> {
    let info = match models.available.get(model_id) {
        Some(info) => info,
        None => return Vec::new(),
    };
    let model_name = info.name.clone();
    let is_current_model = models.current.as_ref() == Some(model_id);
    let options = models.reasoning_effort_options_for(model_id);
    build_effort_arg_items(
        &options,
        models.reasoning_effort,
        is_current_model,
        |option| format!("{model_name} {}", option.id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_shell::sampling::types::ReasoningEffort;

    fn model_with_reasoning(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let mut meta = serde_json::Map::new();
        meta.insert(
            "supportsReasoningEffort".into(),
            serde_json::Value::Bool(true),
        );
        let info = acp::ModelInfo::new(id.clone(), name.to_string())
            .meta(serde_json::Value::Object(meta).as_object().cloned());
        (id, info)
    }

    fn plain_model(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let info = acp::ModelInfo::new(id.clone(), name.to_string());
        (id, info)
    }

    static EMPTY_BUNDLE: crate::app::bundle::BundleState = crate::app::bundle::BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    fn dummy_exec_ctx(models: &ModelState) -> CommandExecCtx<'_> {
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: &EMPTY_BUNDLE,
            screen_mode: crate::app::ScreenMode::Inline,
            billing_surface_visible: true,
            usage_command_visible: true,
            pager_state: crate::settings::PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..crate::settings::PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn split_trailing_token_splits_on_final_whitespace() {
        assert_eq!(
            split_trailing_token("Reasoning X high"),
            Some(("Reasoning X", "high"))
        );
        assert_eq!(
            split_trailing_token("reasoning-x  xhigh"),
            Some(("reasoning-x", "xhigh"))
        );
        // No interior whitespace, so nothing to split off
        assert!(split_trailing_token("reasoning-x-pro").is_none());
    }

    #[test]
    fn empty_query_returns_one_row_per_logical_model() {
        let mut state = ModelState::default();
        let (rid, rinfo) = model_with_reasoning("reasoning-x", "Reasoning X");
        let (pid, pinfo) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(rid, rinfo);
        state.available.insert(pid, pinfo);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            billing_surface_visible: true,
            usage_command_visible: true,
            workflows_available: true,
            saved_workflows: &[],
            workflow_runs: &[],
            screen_mode: crate::app::ScreenMode::Fullscreen,
            current_title: None,
        };
        let items = cmd.suggest_args(&ctx, "").unwrap();
        assert!(
            items.first().is_some_and(|i| i.header && i.display != "Runtimes"),
            "empty /model starts with a runtime section, not a Runtimes banner"
        );
        assert!(
            items
                .iter()
                .filter(|i| i.header)
                .all(|i| !i.display.chars().all(|c| c == '─')),
            "no box-drawing rule row"
        );
        assert_eq!(
            items.iter().filter(|i| !i.header).count(),
            2,
            "models appear once under a runtime header"
        );

        // A reasoning model has a trailing space in insert_text
        // The prompt widget reads it to keep the dropdown open after Enter so the effort sub-menu can render
        let reasoning = items
            .iter()
            .find(|i| i.match_text == "Reasoning X")
            .unwrap();
        assert_eq!(reasoning.insert_text, "Reasoning X ");

        // A plain model has no trailing space, so Enter commits immediately
        let plain = items.iter().find(|i| i.match_text == "Grok 4.5").unwrap();
        assert_eq!(plain.insert_text, "Grok 4.5");
    }

    #[test]
    fn empty_query_groups_models_under_runtime_headers() {
        let mut state = ModelState::default();
        let (a_id, mut a) = plain_model("m5", "M5 Qwen");
        let (a2_id, mut a2) = plain_model("m5-alias", "M5 Qwen localhost");
        let (b_id, mut b) = plain_model("laguna", "Laguna");
        let mut meta_a = serde_json::Map::new();
        meta_a.insert(
            "baseUrl".into(),
            serde_json::Value::String("http://127.0.0.1:11434/v1".into()),
        );
        a.meta = Some(meta_a);
        let mut meta_a2 = serde_json::Map::new();
        meta_a2.insert(
            "baseUrl".into(),
            serde_json::Value::String("http://localhost:11434/v1".into()),
        );
        a2.meta = Some(meta_a2);
        let mut meta_b = serde_json::Map::new();
        meta_b.insert(
            "baseUrl".into(),
            serde_json::Value::String("http://127.0.0.1:8000/v1".into()),
        );
        b.meta = Some(meta_b);
        state.available.insert(a_id, a);
        state.available.insert(a2_id, a2);
        state.available.insert(b_id, b);

let items = build_model_items_grouped(&state);
        let labels: Vec<&str> = items.iter().map(|i| i.display.as_str()).collect();

        assert_ne!(labels[0], "Runtimes");
        assert!(
            items
                .iter()
                .filter(|i| i.header)
                .all(|i| !i.display.contains('—') && !i.display.chars().all(|c| c == '─'))
        );

        let ollama_headers: Vec<_> = labels.iter().filter(|l| l.starts_with("Ollama")).collect();
        assert_eq!(
            ollama_headers.len(),
            1,
            "localhost and 127.0.0.1 must share one Ollama row"
        );
        assert!(ollama_headers[0].contains("(127.0.0.1:11434)"));
        assert!(labels.iter().any(|l| {
            l.starts_with("oMLX") || l.starts_with("OpenAI-compat") || l.starts_with("vLLM")
        }));

        // Find model positions and verify they're preceded by a header
        let m5 = items
            .iter()
            .rposition(|i| i.match_text == "M5 Qwen")
            .unwrap();
        let laguna = items
            .iter()
            .rposition(|i| i.match_text == "Laguna")
            .unwrap();
        assert!(items[m5 - 1].header);
        assert!(items[laguna - 1].header);
        assert!(!items[m5].display.starts_with("  "));
        assert_eq!(items[m5].display, "M5 Qwen");
    }

    #[test]
    fn unknown_origin_header_is_host_port_once() {
        let mut state = ModelState::default();
        let (id, mut info) = plain_model("other", "Other model");
        let mut meta = serde_json::Map::new();
        meta.insert(
            "baseUrl".into(),
            serde_json::Value::String("http://192.168.1.9:9000/v1".into()),
        );
        info.meta = Some(meta);
        state.available.insert(id, info);

        let items = build_model_items_grouped(&state);
        let header = items
            .iter()
            .find(|i| i.header)
            .expect("runtime header");
        assert!(!header.display.starts_with("http://"));
        assert_eq!(header.display.matches("192.168.1.9:9000").count(), 1);
        assert_eq!(header.display, "192.168.1.9:9000");
        assert!(!header.display.contains('—'));
    }

    #[test]
    fn model_row_secondary_text_is_id_and_window() {
        let mut state = ModelState::default();
        let (id, mut info) = plain_model("m5-qwen36-moe", "M5 Pro Qwen");
        let mut meta = serde_json::Map::new();
        meta.insert(
            "totalContextTokens".into(),
            serde_json::Value::Number(200000.into()),
        );
        info.meta = Some(meta);
        state.available.insert(id, info);
        state.current = state.available.keys().next().cloned();

        let items = build_model_items_grouped(&state);
        let row = items.iter().find(|i| !i.header).unwrap();
        assert_eq!(row.display, "M5 Pro Qwen (current)");
        assert!(row.description.contains("m5-qwen36-moe"));
        assert!(row.description.contains("200k"));
        assert!(!row.display.starts_with(' '));
    }

    #[test]
    fn compact_window_prefers_1024_then_1000() {
        assert_eq!(compact_context_window(8192), "8k");
        assert_eq!(compact_context_window(200000), "200k");
        assert_eq!(compact_context_window(131072), "128k");
        assert_eq!(compact_context_window(512), "512");
    }

    #[test]
    fn name_equal_id_does_not_repeat_key() {
        let mut state = ModelState::default();
        let (id, mut info) = plain_model("local", "local");
        let mut meta = serde_json::Map::new();
        meta.insert(
            "totalContextTokens".into(),
            serde_json::Value::Number(8192.into()),
        );
        info.meta = Some(meta);
        state.available.insert(id, info);
        let items = build_model_items_flat(&state);
        assert_eq!(items[0].description, "8k");
    }

    #[test]
    fn trailing_space_after_reasoning_model_enters_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            billing_surface_visible: true,
            usage_command_visible: true,
            workflows_available: true,
            saved_workflows: &[],
            workflow_runs: &[],
            screen_mode: crate::app::ScreenMode::Fullscreen,
            current_title: None,
        };
        // The args query has a trailing space, so this is the effort phase
        // Items come out ordered xhigh to low (strongest first) per EFFORT_LEVELS
        let items = cmd.suggest_args(&ctx, "Reasoning X ").unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].insert_text, "Reasoning X xhigh");
        assert_eq!(items[1].insert_text, "Reasoning X high");
        assert_eq!(items[2].insert_text, "Reasoning X medium");
        assert_eq!(items[3].insert_text, "Reasoning X low");
        // Display is just the level so the user sees a clean column.
        assert_eq!(items[0].display, "xhigh");
        // match_text carries the sort-key prefix that forces the matcher's alphabetical tiebreak to render rows in EFFORT_LEVELS order
        assert!(items[0].match_text.starts_with("a "));
        assert!(items[3].match_text.starts_with("d "));
    }

    #[test]
    fn partial_effort_query_still_in_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            billing_surface_visible: true,
            usage_command_visible: true,
            workflows_available: true,
            saved_workflows: &[],
            workflow_runs: &[],
            screen_mode: crate::app::ScreenMode::Fullscreen,
            current_title: None,
        };
        // Still in effort phase; the matcher upstream narrows to high and xhigh
        let items = cmd.suggest_args(&ctx, "Reasoning X h").unwrap();
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn partial_model_query_stays_in_model_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            billing_surface_visible: true,
            usage_command_visible: true,
            workflows_available: true,
            saved_workflows: &[],
            workflow_runs: &[],
            screen_mode: crate::app::ScreenMode::Fullscreen,
            current_title: None,
        };
        // No trailing space: the user is still typing the model name
        let items = cmd.suggest_args(&ctx, "Reason").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].insert_text, "Reasoning X ");
    }

    #[test]
    fn run_parses_model_plus_effort_when_supported() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X xhigh");
        match result {
            CommandResult::Action(Action::SwitchModel { model_id, effort }) => {
                assert_eq!(model_id.0.as_ref(), "reasoning-x");
                assert_eq!(effort, Some(ReasoningEffort::Xhigh));
            }
            other => panic!("expected SwitchModel with effort, got {other:?}"),
        }
    }

    #[test]
    fn run_rejects_unoffered_effort_with_effort_error_not_unknown_model() {
        // Regression: previously `resolve_effort_token_for` returned None and the handler fell through to `Unknown model: Reasoning X none`
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X none");
        match result {
            CommandResult::Error(msg) => {
                assert!(
                    msg.contains("unknown effort level 'none'"),
                    "expected effort error, got {msg}"
                );
                assert!(
                    msg.contains("use one of:"),
                    "expected offered levels in message, got {msg}"
                );
                assert!(
                    !msg.to_lowercase().contains("unknown model"),
                    "must not misreport as unknown model: {msg}"
                );
                let offered = msg.split_once("; ").map(|(_, r)| r).unwrap_or("");
                assert!(
                    !offered.contains("none"),
                    "must not list none as offered: {msg}"
                );
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn run_prefers_full_multi_word_model_name_over_prefix_plus_effort() {
        // The catalog has both "Grok" (reasoning) and "Grok 4.5"
        // `/model Grok 4.5` must select the full name, not treat "4.5" as an effort on "Grok"
        let mut state = ModelState::default();
        let (short_id, short_info) = model_with_reasoning("grok", "Grok");
        let (long_id, long_info) = model_with_reasoning("grok-4.5", "Grok 4.5");
        state.available.insert(short_id, short_info);
        state.available.insert(long_id.clone(), long_info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, long_id);
            }
            other => panic!("expected SetDefaultModel(Grok 4.5), got {other:?}"),
        }
    }

    #[test]
    fn run_rejects_effort_for_non_reasoning_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5 high");
        // Falls through to "is the whole string a model name?", which it isn't, so we get an Unknown error
        assert!(matches!(result, CommandResult::Error(_)));
    }

    /// The bare `/model <name>` form dispatches `Action::SetDefaultModel(<ModelId>)` instead of the legacy `Action::SwitchModel { effort: None }`.
    /// The dispatcher routes it through both `Effect::SwitchModel` (session mutation) and `Effect::PersistSetting` (next-session default).
    /// The payload is the typed `acp::ModelId` (resolved at the slash boundary), not a String.
    #[test]
    fn run_bare_model_name_dispatches_set_default_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }

    /// Case-insensitive matching against the catalog: `/model grok 4.5` resolves to the same `ModelId` as `/model Grok 4.5`.
    #[test]
    fn run_set_default_model_resolves_case_insensitively() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }
}
