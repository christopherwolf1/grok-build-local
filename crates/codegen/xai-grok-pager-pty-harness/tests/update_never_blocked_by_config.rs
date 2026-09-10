//! Local-first: `grok update` is a no-op (no xAI phone-home). A corrupt
//! config.toml must not change that outcome.
//!
//! A local server serves the binary's own version as the channel pointer, so a healthy run exits 0 ("already up to date").
//! A run with a corrupt config must exit 0 too; reintroducing a config `?` fails exactly that run.
//! The pointer must equal the current version: the installer converges in both directions, so an older pointer triggers a downgrade attempt.

use std::io::{Read, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};

use xai_grok_pager_pty_harness::pager_binary;

/// Spawn a local server that answers every request with the channel pointer body.
fn spawn_pointer_server(body: Arc<Mutex<String>>) -> (std::net::TcpListener, String) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let serving = listener.try_clone().unwrap();
    std::thread::spawn(move || {
        for stream in serving.incoming() {
            let Ok(mut stream) = stream else { return };
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let version = body.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    version.len(),
                    version
                )
                .as_bytes(),
            );
        }
    });
    (listener, base)
}

/// Run `grok update` in an isolated home against the local pointer base.
fn run_update(base: &str, config_toml: &str, extra_args: &[&str]) -> std::process::Output {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("config.toml"), config_toml).unwrap();
    Command::new(pager_binary().expect("resolve pager binary"))
        .arg("update")
        .args(extra_args)
        .env_clear()
        .env("HOME", home.path())
        .env("GROK_HOME", home.path())
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("GROK_CLI_BASE_URL", base)
        .output()
        .expect("spawn grok update")
}

/// Local-only update path: always exits 0 and does not phone home, even
/// when `config.toml` is corrupt.
#[test]
fn corrupt_config_never_changes_update_outcome() {
    let body = Arc::new(Mutex::new("0.0.1".to_owned()));
    let (_listener, base) = spawn_pointer_server(body.clone());

    let valid = run_update(&base, "[cli]\n", &[]);
    assert!(
        valid.status.success(),
        "local-only grok update must exit 0\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&valid.stdout),
        String::from_utf8_lossy(&valid.stderr)
    );
    let valid_err = String::from_utf8_lossy(&valid.stderr);
    assert!(
        valid_err.contains("disabled in local-only mode"),
        "expected local-only copy, stderr:\n{valid_err}"
    );

    let corrupt = run_update(&base, "this is not toml {{{[[[", &[]);
    assert!(
        corrupt.status.success(),
        "a corrupt config.toml must not block grok update\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&corrupt.stdout),
        String::from_utf8_lossy(&corrupt.stderr)
    );
}
