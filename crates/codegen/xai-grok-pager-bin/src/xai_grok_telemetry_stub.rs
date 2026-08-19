//! Telemetry stub module - no-op implementations for local-only mode

pub mod debug_log {
    pub const RMCP_SSE_NOISE_TARGET: &str = "rmcp::sse";
    pub fn install_firehose(_registry: tracing_subscriber::registry::Registry, _app_entrypoint: &str) {}
    pub fn flush() {}
}

pub mod sampling_log {
    #[derive(Debug)]
    pub struct SamplingLayer;
    impl<S: tracing::Subscriber + std::panic::UnwindSafe> tracing_subscriber::layer::Layer<S> for SamplingLayer {}
    
    pub fn layer() -> SamplingLayer { SamplingLayer }
}

pub mod instrumentation {
    #[derive(Debug)]
    pub struct InstrumentationLayer;
    impl<S: tracing::Subscriber + std::panic::UnwindSafe> tracing_subscriber::layer::Layer<S> for InstrumentationLayer {}
    
    pub fn layer() -> InstrumentationLayer { InstrumentationLayer }
    pub fn install_panic_hook() {}
}

pub mod hooks_log {
    #[derive(Debug)]
    pub struct HooksLayer;
    impl<S: tracing::Subscriber + std::panic::UnwindSafe> tracing_subscriber::layer::Layer<S> for HooksLayer {}
    
    pub fn layer() -> HooksLayer { HooksLayer }
}

pub mod otel_layer {
    pub struct OtelClientInfo {
        pub client_name: &'static str,
        pub client_version: &'static str,
        pub service_version: &'static str,
        pub app_entrypoint: &'static str,
    }
    
    #[derive(Debug)]
    pub struct OtelLayer;
    impl<S: tracing::Subscriber + std::panic::UnwindSafe> tracing_subscriber::layer::Layer<S> for OtelLayer {}
    
    pub fn build_otel_layer(_info: OtelClientInfo, _config: ()) -> OtelLayer { OtelLayer }
    
    pub struct OtelGuard;
    impl Drop for OtelGuard {
        fn drop(&mut self) {}
    }
    
    pub fn otel_guard() -> OtelGuard { OtelGuard }
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
    
    pub fn init(_config: Config) -> SentryGuard { SentryGuard }
    pub fn flush_on_shutdown() {}
}

pub mod external {
    pub mod config {
        pub struct ExternalClientInfo {
            pub service_version: String,
            pub client_version: String,
            pub app_entrypoint: String,
        }
    }
    
    pub fn init(_config: config::ExternalClientInfo) {}
}

pub mod startup {
    pub fn mark_process_start() {}
}

pub mod session_ctx {
    pub const CLI_DRAIN: u8 = 1;
    pub async fn drain_pending(_drain: u8) {}
}