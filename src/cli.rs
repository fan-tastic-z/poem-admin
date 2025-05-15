use std::{path::PathBuf, sync::Arc};

use crate::{
    auth::auth::JWT,
    config::config::{Config, LoadConfigResult, load_config},
    domain::{
        models::account::{AccountName, AccountPassword},
        ports::SysService,
        services::Service,
    },
    errors::Error,
    input::http::http_server::{self, make_acceptor_and_advertise_addr},
    output::db::db::Db,
    utils::{
        num_cpus,
        password_hash::compute_password_hash,
        runtime::{Runtime, make_runtime},
        telemetry,
    },
};
use clap::ValueHint;
use error_stack::{Result, ResultExt};
use rust_embed::RustEmbed;

#[derive(Clone)]
pub struct Ctx<S: SysService + Send + Sync + 'static> {
    pub sys_service: Arc<S>,
    pub jwt: Arc<JWT>,
    pub config: Arc<Config>,
}

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
        log::info!("server is starting with config: {config:#?}");
        let server_runtime = make_server_runtime();
        server_runtime.block_on(run_server(&server_runtime, config))
    }
}

async fn run_server(server_rt: &Runtime, config: Config) -> Result<(), Error> {
    let make_error = || Error::Message("failed to start server".to_string());
    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();
    let (acceptor, advertise_addr) = make_acceptor_and_advertise_addr(
        &config.server.listen_addr,
        config.server.advertise_addr.as_deref(),
    )
    .await
    .change_context_lazy(make_error)?;
    let jwt = JWT::new(&config.auth.jwt.secret);
    let db = Db::new(&config).await.change_context_lazy(make_error)?;
    let sys_service = Service::new(db);

    let ctx = Ctx {
        sys_service: Arc::new(sys_service),
        jwt: Arc::new(jwt),
        config: Arc::new(config),
    };

    let server = http_server::start_server(server_rt, shutdown_rx, ctx, acceptor, advertise_addr)
        .await
        .change_context_lazy(|| {
            Error::Message("A fatal error has occurred in server process.".to_string())
        })?;

    ctrlc::set_handler(move || shutdown_tx.shutdown()).change_context_lazy(|| {
        Error::Message("failed to setup ctrl-c signal handle".to_string())
    })?;

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

fn make_init_data_runtime() -> Runtime {
    make_runtime("init_data_runtime", "init_data_thread", 1)
}

#[derive(RustEmbed)]
#[folder = "init_data/"]
struct SqlFiles;

#[derive(Debug, clap::Parser)]
pub struct CommandInitData {
    #[clap(short, long, help = "Path to config file", value_hint = ValueHint::FilePath)]
    config_file: PathBuf,
}

impl CommandInitData {
    pub fn run(self) -> Result<(), Error> {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);
        let LoadConfigResult { config, warnings } = load_config(self.config_file)?;
        let telemetry_runtime = make_telemetry_runtime();
        let mut drop_guards =
            telemetry::init(&telemetry_runtime, "init-data", config.telemetry.clone());
        drop_guards.push(Box::new(telemetry_runtime));
        for warning in warnings {
            log::warn!("{warning}");
        }
        let init_data_runtime = make_init_data_runtime();
        init_data_runtime.block_on(run_init_data(config))
    }
}

async fn run_init_data(config: Config) -> Result<(), Error> {
    let make_error = || Error::Message("failed to init data".to_string());
    let db = Db::new(&config).await.change_context_lazy(make_error)?;
    let mut tx = db.pool.begin().await.change_context_lazy(make_error)?;

    let mut sql_files: Vec<_> = SqlFiles::iter().collect();
    sql_files.sort();

    for file_path in sql_files {
        if file_path.ends_with(".sql") {
            let content = SqlFiles::get(&file_path)
                .ok_or_else(|| Error::Message(format!("failed to read sql file: {}", file_path)))?;

            let sql = std::str::from_utf8(&content.data).change_context_lazy(|| {
                Error::Message(format!(
                    "failed to convert sql file to string: {}",
                    file_path
                ))
            })?;

            let statements = sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty());
            for statement in statements {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .change_context_lazy(|| {
                        Error::Message(format!("failed to execute sql: {}", file_path))
                    })?;
            }
        }
    }
    tx.commit().await.change_context_lazy(make_error)?;
    log::info!("init data success");
    Ok(())
}

#[derive(Debug, clap::Parser)]
pub struct CreateSuperUser {
    #[clap(short, long, help = "Path to config file", value_hint = ValueHint::FilePath)]
    config_file: PathBuf,
    #[clap(short, long, help = "Password")]
    password: String,
}

impl CreateSuperUser {
    pub fn run(self) -> Result<(), Error> {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);
        let LoadConfigResult { config, warnings } = load_config(self.config_file)?;
        let telemetry_runtime = make_telemetry_runtime();
        let mut drop_guards =
            telemetry::init(&telemetry_runtime, "init-data", config.telemetry.clone());
        drop_guards.push(Box::new(telemetry_runtime));
        for warning in warnings {
            log::warn!("{warning}");
        }
        let init_data_runtime = make_init_data_runtime();
        init_data_runtime.block_on(run_create_super_user(config, self.password))
    }
}

async fn run_create_super_user(config: Config, password: String) -> Result<(), Error> {
    let make_error = || Error::Message("failed to create super user".to_string());
    let password_hash = compute_password_hash(&password).change_context_lazy(make_error)?;
    let db = Db::new(&config).await.change_context_lazy(make_error)?;
    let mut tx = db.pool.begin().await.change_context_lazy(make_error)?;
    db.save_super_user(
        &mut tx,
        &AccountName::try_new("admin").change_context_lazy(make_error)?,
        &AccountPassword::try_new(password_hash).change_context_lazy(make_error)?,
    )
    .await
    .change_context_lazy(make_error)?;
    tx.commit().await.change_context_lazy(make_error)?;
    log::info!("create super user success");
    Ok(())
}
