use logforth::{
    append::{
        self,
        rolling_file::{RollingFileBuilder, Rotation},
    },
    diagnostic::{FastraceDiagnostic, StaticDiagnostic},
    filter::{EnvFilter, env_filter::EnvFilterBuilder},
    layout,
};

use crate::config::config::TelemetryConfig;

use super::runtime::Runtime;

pub fn init(
    rt: &Runtime,
    service_name: &'static str,
    config: TelemetryConfig,
) -> Vec<Box<dyn Send + Sync + 'static>> {
    let mut drop_guards = vec![];
    drop_guards.extend(init_logs(rt, service_name, &config));
    drop_guards
}

fn init_logs(
    rt: &Runtime,
    service_name: &'static str,
    config: &TelemetryConfig,
) -> Vec<Box<dyn Send + Sync + 'static>> {
    let _ = rt;
    let static_diagnostic = StaticDiagnostic::default();

    let mut drop_guards: Vec<Box<dyn Send + Sync + 'static>> = Vec::new();
    let mut builder = logforth::builder();

    // file appender
    if let Some(file) = &config.logs.file {
        let (rolling, guard) = RollingFileBuilder::new(&file.dir)
            .layout(layout::JsonLayout::default())
            .rotation(Rotation::Hourly)
            .filename_prefix(service_name)
            .filename_suffix("log")
            .max_log_files(file.max_files)
            .build()
            .expect("failed to initialize rolling file appender");
        drop_guards.push(guard);
        builder = builder.dispatch(|b| {
            b.filter(make_rust_log_filter(&file.filter))
                .diagnostic(FastraceDiagnostic::default())
                .diagnostic(static_diagnostic.clone())
                .append(rolling)
        });
    }

    // stderr appender
    if let Some(stderr) = &config.logs.stderr {
        builder = builder.dispatch(|b| {
            b.filter(make_rust_log_filter_with_default_env(&stderr.filter))
                .diagnostic(FastraceDiagnostic::default())
                .diagnostic(static_diagnostic.clone())
                .append(append::Stderr::default())
        });
    }

    // apply returns err if already set; ignored
    let _ = builder.try_apply();

    drop_guards
}

fn make_rust_log_filter(filter: &str) -> EnvFilter {
    let builder = EnvFilterBuilder::new()
        .try_parse(filter)
        .unwrap_or_else(|_| panic!("failed to parse filter: {filter}"));
    EnvFilter::new(builder)
}

fn make_rust_log_filter_with_default_env(filter: &str) -> EnvFilter {
    if let Ok(filter) = std::env::var("RUST_LOG") {
        make_rust_log_filter(&filter)
    } else {
        make_rust_log_filter(filter)
    }
}
