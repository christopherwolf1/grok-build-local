//! Telemetry no-ops at the pager-bin seam. Hosted Mixpanel/OTEL crates stay in the
//! workspace; this binary does not link them.
#![allow(dead_code)]

pub mod debug_log {
    pub fn flush() {}
}

pub mod otel_layer {
    pub struct OtelGuard;
    impl Drop for OtelGuard {
        fn drop(&mut self) {}
    }
    pub fn otel_guard() -> OtelGuard {
        OtelGuard
    }
    pub fn shutdown_otel() {}
}

pub mod sentry {
    pub struct Config {
        pub client: &'static str,
        pub client_version: &'static str,
        pub release: &'static str,
        pub disabled: bool,
    }
    pub struct SentryGuard;
    impl Drop for SentryGuard {
        fn drop(&mut self) {}
    }
    pub fn init(_config: Config) -> SentryGuard {
        SentryGuard
    }
    pub fn flush_on_shutdown() {}
}

pub mod startup {
    pub fn mark_process_start() {}
}

pub mod span_profile {
    pub fn finalize() -> Option<std::path::PathBuf> {
        None
    }
}

pub mod process_info {
    #[derive(Clone, Copy)]
    pub enum Entrypoint {
        Embedded,
        Leader,
        Cli,
        Headless,
        Workspace,
    }
    #[derive(Clone, Copy)]
    pub enum Interactivity {
        Interactive,
        Unattended,
    }
    #[derive(Clone, Copy)]
    pub enum LeaderMode {
        Attached,
        Standalone,
    }
    #[derive(Clone, Copy)]
    pub enum ReleaseChannel {
        Stable,
        Alpha,
        Unknown,
    }
    impl ReleaseChannel {
        pub fn from_label(label: &str) -> ReleaseChannel {
            match label.trim().trim_start_matches('[').trim_end_matches(']') {
                "stable" => ReleaseChannel::Stable,
                "alpha" => ReleaseChannel::Alpha,
                _ => ReleaseChannel::Unknown,
            }
        }
    }
    #[derive(Clone, Copy)]
    pub struct ProcessIdentity {
        pub entrypoint: Entrypoint,
        pub leader: LeaderMode,
        pub interactivity: Interactivity,
    }
    pub fn set_identity(_identity: ProcessIdentity) {}
    pub fn set_release_channel(_channel: ReleaseChannel) {}
}
