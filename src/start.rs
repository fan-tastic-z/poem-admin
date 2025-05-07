use std::path::PathBuf;

use clap::ValueHint;
use error_stack::{Result, ResultExt};

use crate::{
    Error,
    config::config::{Config, LoadConfigResult, load_config},
    server::http_server::{self, make_acceptor_and_advertise_addr},
    utils::{
        num_cpus,
        runtime::{Runtime, make_runtime},
        telemetry,
    },
};

#[derive(Debug, clap::Parser)]
pub struct CommandStart {
    #[clap(short, long, help = "Path to config file", value_hint = ValueHint::FilePath)]
    config_file: PathBuf,
}

impl CommandStart {
    pub fn run(self) -> Result<(), Error> {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);
        let LoadConfigResult { config, warnings } = load_config(self.config_file)?;
        let telemetry_runtime = make_telemetry_runtime();
        let mut drop_guards =
            telemetry::init(&telemetry_runtime, "poem-admin", config.telemetry.clone());
        drop_guards.push(Box::new(telemetry_runtime));
        for warning in warnings {
            log::warn!("{warning}");
        }

        log::info!("insight is starting with config: {config:#?}");
        let server_runtime = make_server_runtime();
        server_runtime.block_on(run_server(&server_runtime, config))
    }
}

async fn run_server(server_rt: &Runtime, config: Config) -> Result<(), Error> {
    let make_error = || Error("failed to start server".to_string());
    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();
    let (acceptor, advertise_addr) = make_acceptor_and_advertise_addr(
        &config.server.listen_addr,
        config.server.advertise_addr.as_deref(),
    )
    .await
    .change_context_lazy(make_error)?;

    let server = http_server::start_server(server_rt, shutdown_rx, acceptor, advertise_addr)
        .await
        .change_context_lazy(|| {
            Error("A fatal error has occurred in server process.".to_string())
        })?;

    ctrlc::set_handler(move || shutdown_tx.shutdown())
        .change_context_lazy(|| Error("failed to setup ctrl-c signal handle".to_string()))?;

    server.await_shutdown().await;
    Ok(())
}

fn make_server_runtime() -> Runtime {
    let parallelism = num_cpus().get();
    make_runtime("server_runtime", "server_thread", parallelism)
}

fn make_telemetry_runtime() -> Runtime {
    make_runtime("telemetry_runtime", "telemetry_thread", 1)
}
