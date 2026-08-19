//! Local-only stub module for update functionality
//! Provides no-op implementations for local-first usage

pub const VERSION: &str = env!("VERSION_WITH_COMMIT");

pub fn channel_label() -> &'static str {
    "local"
}

pub fn channel_name() -> &'static str {
    "local"
}

pub fn write_version_cache(_version: &str) {}

#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub deployment_key: Option<String>,
    pub channel: String,
    pub npm_registry: Option<String>,
}

impl UpdateConfig {
    pub fn from_environment(_env: &xai_grok_shell::env::GrokBuildEnvironment) -> Self {
        Self {
            deployment_key: None,
            channel: "stable".to_string(),
            npm_registry: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Current,
}

pub fn auto_update(_cfg: &UpdateConfig) -> Result<UpdateStatus, anyhow::Error> {
    Ok(UpdateStatus::Current)
}

pub fn enforce_version_policy_or_exit(_allow_prerelease: bool) {}

pub fn display_version_with_commit(version: &str, _channel_label: &str) -> String {
    format!("v{}", version)
}

pub fn version_text(channel_label: &str) -> String {
    format!("v{} {}", env!("VERSION_WITH_COMMIT"), channel_label)
}

pub mod auto_update {
    use super::*;
    use tracing_subscriber::layer::Layer;
    
    #[derive(Debug, Clone, Copy)]
    pub enum UpdateStatus {
        Current,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub enum CliUpdateTrigger {
        UserCommand,
        AutoBackground,
    }
    
    pub enum UpdateAvailable {
        None,
    }
    
    #[derive(Debug, Default)]
    pub struct UpdateStatusResult {
        pub current: Option<String>,
    }
    
    #[derive(Debug, Default)]
    pub struct BackgroundCheckResult {
        pub update: Option<String>,
        pub download: Option<std::process::Command>,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub enum UpdateRunMode {
        NonBlocking,
        Blocking,
    }
    
    pub async fn run_update_if_available(
        _mode: UpdateRunMode,
        _force: bool,
        _trigger: CliUpdateTrigger,
        _cfg: &UpdateConfig,
    ) -> Result<Option<UpdateAvailable>, anyhow::Error> {
        Ok(None)
    }
    
    pub async fn check_update_status(_cfg: &UpdateConfig) -> Result<UpdateStatusResult, anyhow::Error> {
        Ok(UpdateStatusResult { current: None })
    }
    
    pub fn print_update_status(_status: &UpdateStatusResult, _json: bool) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    pub fn apply_channel_switch(_channel: Option<&str>, _cfg: &mut UpdateConfig) -> anyhow::Result<()> {
        Ok(())
    }
    
    pub async fn check_update_background(_cfg: &UpdateConfig) -> Result<BackgroundCheckResult, anyhow::Error> {
        Ok(Default::default())
    }
    
    pub async fn run_update(
        _force_reinstall: bool,
        _version: Option<&str>,
        _channel: Option<&str>,
        _cfg: &mut UpdateConfig,
        _trigger: CliUpdateTrigger,
    ) -> Result<Result<String, anyhow::Error>, anyhow::Error> {
        Ok(Ok("Update disabled in local mode".to_string()))
    }
    
    pub async fn ensure_latest_on_disk(_cfg: &UpdateConfig) -> Result<bool, anyhow::Error> {
        Ok(true)
    }
}

/// Stub for managed install detection
pub fn installed_grok_version() -> Option<String> {
    None
}

/// Stub for disk version
pub fn disk_version_for_installer(_installer: &str, _cfg: &UpdateConfig) -> Option<String> {
    None
}