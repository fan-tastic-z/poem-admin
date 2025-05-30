use mea::shutdown;
use poem_admin::{
    auth::jwt::JWT,
    cli::Ctx,
    config::settings::{
        Auth, Config, Database, Jwt, LogsConfig, Server, StderrAppenderConfig, TelemetryConfig,
    },
    domain::{
        models::{
            account::{AccountName, AccountPassword, CreateAccountRequest},
            organization::OrganizationName,
            role::RoleName,
        },
        ports::SysService,
        services::Service,
    },
    input::http::http_server::{make_acceptor_and_advertise_addr, start_server},
    output::db::database::Db,
    utils::{password_hash::compute_password_hash, runtime::make_runtime},
};
use reqwest::Client;
use serde_json::Value;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tokio::time::sleep;
use uuid::Uuid;

#[allow(dead_code)] // 这些字段在测试框架中是必要的，即使看起来未使用
pub struct TestEnvironment {
    pub container: ContainerAsync<Postgres>,
    pub ctx: Ctx<Service<Db>>,
    pub base_url: String,
    pub client: Client,
    pub db: Db,
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
    pub server_addr: Option<SocketAddr>,
    pub shutdown_tx: Option<shutdown::ShutdownSend>,
}

impl TestEnvironment {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let container = Postgres::default().start().await?;

        // 等待容器启动
        sleep(Duration::from_secs(3)).await;

        // 获取容器端口
        let host_port = container.get_host_port_ipv4(5432).await?;

        // 构建测试配置
        let config = Config {
            server: Server {
                listen_addr: "127.0.0.1:0".to_string(),
                advertise_addr: None,
            },
            auth: Auth {
                jwt: Jwt {
                    secret: "dGVzdF9zZWNyZXRfa2V5X2Zvcl9pbnRlZ3JhdGlvbl90ZXN0c19vbmx5X2RvX25vdF91c2VfaW5fcHJvZHVjdGlvbg==".to_string(), // Base64编码的测试密钥
                    expiration: 3600, // 1 hour for tests
                },
            },
            database: Database {
                host: "127.0.0.1".to_string(),
                port: host_port,
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                database_name: "postgres".to_string(), // 恢复使用postgres数据库
            },
            telemetry: TelemetryConfig {
                logs: LogsConfig {
                    file: None,
                    stderr: Some(StderrAppenderConfig {
                        filter: "ERROR".to_string(),
                    }),
                },
            },
        };

        // 初始化数据库连接
        let db = Db::new(&config)
            .await
            .map_err(|e| format!("Failed to connect to database: {:?}", e))?;

        // 运行数据库迁移
        sqlx::migrate!("./migrations")
            .run(&db.pool)
            .await
            .map_err(|e| format!("Failed to run migrations: {:?}", e))?;

        // 初始化测试数据
        Self::init_test_data(&db).await?;

        let jwt = JWT::new(&config.auth.jwt.secret);
        let sys_service = Service::new(db.clone());

        let ctx = Ctx {
            sys_service: Arc::new(sys_service),
            jwt: Arc::new(jwt),
            config: Arc::new(config),
        };

        let client = Client::new();

        Ok(Self {
            container,
            ctx,
            base_url: "http://127.0.0.1:9000".to_string(),
            client,
            db,
            server_handle: None,
            server_addr: None,
            shutdown_tx: None,
        })
    }

    async fn init_test_data(db: &Db) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = db.pool.begin().await?;

        // 先清理所有测试数据
        sqlx::query("DELETE FROM role_menu")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM account").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM role").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM organization")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM menu").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM operation_log")
            .execute(&mut *tx)
            .await?;

        // 插入测试菜单数据，让数据库自动生成ID
        let menu_user_mgmt_id: i64 = sqlx::query_scalar(
            "INSERT INTO menu (name, parent_id, parent_name, order_index) VALUES ('用户管理', -1, '', 10) RETURNING id"
        )
        .fetch_one(&mut *tx)
        .await?;

        let menu_user_setting_id: i64 = sqlx::query_scalar(
            "INSERT INTO menu (name, parent_id, parent_name, order_index) VALUES ('用户设置', $1, '用户管理', 10) RETURNING id"
        )
        .bind(menu_user_mgmt_id)
        .fetch_one(&mut *tx)
        .await?;

        let menu_role_setting_id: i64 = sqlx::query_scalar(
            "INSERT INTO menu (name, parent_id, parent_name, order_index) VALUES ('角色设置', $1, '用户管理', 20) RETURNING id"
        )
        .bind(menu_user_mgmt_id)
        .fetch_one(&mut *tx)
        .await?;

        let menu_operation_log_id: i64 = sqlx::query_scalar(
            "INSERT INTO menu (name, parent_id, parent_name, order_index) VALUES ('用户操作日志', $1, '用户管理', 30) RETURNING id"
        )
        .bind(menu_user_mgmt_id)
        .fetch_one(&mut *tx)
        .await?;

        // 插入测试组织数据
        let org_company_id: i64 = sqlx::query_scalar(
            "INSERT INTO organization (name, parent_id, parent_name) VALUES ('总公司', -1, '') RETURNING id"
        )
        .fetch_one(&mut *tx)
        .await?;

        let _org_tech_id: i64 = sqlx::query_scalar(
            "INSERT INTO organization (name, parent_id, parent_name) VALUES ('技术部', $1, '总公司') RETURNING id"
        )
        .bind(org_company_id)
        .fetch_one(&mut *tx)
        .await?;

        // 插入测试角色数据
        let role_admin_id: i64 = sqlx::query_scalar(
            "INSERT INTO role (name, description, created_by, created_by_name, is_deletable) VALUES ('超级管理员', '系统超级管理员', 0, 'system', false) RETURNING id"
        )
        .fetch_one(&mut *tx)
        .await?;

        let role_user_id: i64 = sqlx::query_scalar(
            "INSERT INTO role (name, description, created_by, created_by_name, is_deletable) VALUES ('普通用户', '普通用户角色', 0, 'system', true) RETURNING id"
        )
        .fetch_one(&mut *tx)
        .await?;

        // 插入角色菜单关联数据
        sqlx::query(
            r#"
            INSERT INTO role_menu (role_id, role_name, menu_id, menu_name) VALUES
            ($1, '超级管理员', $2, '用户管理'),
            ($1, '超级管理员', $3, '用户设置'),
            ($1, '超级管理员', $4, '角色设置'),
            ($1, '超级管理员', $5, '用户操作日志'),
            ($6, '普通用户', $3, '用户设置')
        "#,
        )
        .bind(role_admin_id)
        .bind(menu_user_mgmt_id)
        .bind(menu_user_setting_id)
        .bind(menu_role_setting_id)
        .bind(menu_operation_log_id)
        .bind(role_user_id)
        .execute(&mut *tx)
        .await?;

        // 创建默认管理员账户
        let admin_password = compute_password_hash("admin123")
            .map_err(|e| format!("Failed to hash password: {:?}", e))?;
        let _admin_id: i64 = sqlx::query_scalar(
            "INSERT INTO account (name, email, password, organization_id, organization_name, role_id, role_name, is_deletable) VALUES ('admin', 'admin@example.com', $1, $2, '总公司', $3, '超级管理员', false) RETURNING id"
        )
        .bind(admin_password)
        .bind(org_company_id)
        .bind(role_admin_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        log::info!("Test data initialized successfully");
        Ok(())
    }

    #[allow(dead_code)] // 测试框架方法，可能在特定测试中使用
    pub async fn get_admin_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let login_request = poem_admin::domain::models::auth::LoginRequest {
            username: AccountName::try_new("admin".to_string())
                .map_err(|e| format!("Invalid username: {:?}", e))?,
            password: AccountPassword::try_new("admin123".to_string())
                .map_err(|e| format!("Invalid password: {:?}", e))?,
        };

        let account = self
            .ctx
            .sys_service
            .login(&login_request)
            .await
            .map_err(|e| format!("Login failed: {:?}", e))?;

        let token = self
            .ctx
            .jwt
            .generate_token(3600, account.id, serde_json::Map::new())
            .map_err(|e| format!("Token generation failed: {:?}", e))?;

        Ok(token)
    }

    #[allow(dead_code)] // 测试框架方法，可能在特定测试中使用
    pub async fn create_test_user(
        &self,
    ) -> Result<(String, String, i64), Box<dyn std::error::Error>> {
        let username = format!("test{}", &Uuid::new_v4().to_string()[..6]);
        let password = "test_password_123";

        // 动态查询组织和角色ID
        let org_id: i64 =
            sqlx::query_scalar("SELECT id FROM organization WHERE name = '总公司' LIMIT 1")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| format!("Failed to find organization: {:?}", e))?;

        let role_id: i64 =
            sqlx::query_scalar("SELECT id FROM role WHERE name = '普通用户' LIMIT 1")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| format!("Failed to find role: {:?}", e))?;

        let request = CreateAccountRequest::new(
            AccountName::try_new(username.clone())
                .map_err(|e| format!("Invalid username: {:?}", e))?,
            AccountPassword::try_new(password.to_string())
                .map_err(|e| format!("Invalid password: {:?}", e))?,
            org_id,
            OrganizationName::try_new("总公司".to_string())
                .map_err(|e| format!("Invalid org name: {:?}", e))?,
            role_id,
            RoleName::try_new("普通用户".to_string())
                .map_err(|e| format!("Invalid role name: {:?}", e))?,
        );

        let user_id = self
            .ctx
            .sys_service
            .create_account(request, 1)
            .await
            .map_err(|e| format!("Failed to create test user: {:?}", e))?;

        Ok((username, password.to_string(), user_id))
    }

    /// 启动HTTP服务器用于API测试
    #[allow(dead_code)] // API测试框架方法
    pub async fn start_http_server(&mut self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        // 创建shutdown信号
        let (shutdown_tx, shutdown_rx) = shutdown::new_pair();

        // 使用随机端口启动服务器
        let listen_addr = "127.0.0.1:0";
        let (acceptor, advertise_addr) =
            make_acceptor_and_advertise_addr(listen_addr, None).await?;

        // 在blocking context中创建运行时并启动服务器
        let ctx = self.ctx.clone();
        let (server_state, rt) = tokio::task::spawn_blocking(move || {
            let rt = make_runtime("test_server", "test_thread", 1);
            let server_state = rt.block_on(start_server(
                &rt,
                shutdown_rx,
                ctx,
                acceptor,
                advertise_addr,
            ))?;
            Ok::<_, std::io::Error>((server_state, rt))
        })
        .await
        .map_err(|e| format!("Join error: {}", e))?
        .map_err(|e| format!("Server start error: {}", e))?;

        let server_addr = server_state.advertise_addr();

        // 更新base_url
        self.base_url = format!("http://{}", server_addr);
        self.server_addr = Some(server_addr);
        self.shutdown_tx = Some(shutdown_tx);

        // 启动服务器任务，确保runtime不会被drop
        let server_handle = tokio::spawn(async move {
            server_state.await_shutdown().await;
            // 确保runtime在服务器关闭后才被drop
            drop(rt);
        });
        self.server_handle = Some(server_handle);

        // 等待服务器准备就绪
        wait_for_server_ready(&self.base_url, 30).await?;

        log::info!("Test HTTP server started on {}", server_addr);
        Ok(server_addr)
    }

    /// 停止HTTP服务器
    #[allow(dead_code)] // API测试框架方法
    pub async fn stop_http_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.shutdown();
        }

        if let Some(server_handle) = self.server_handle.take() {
            // 等待服务器优雅关闭，但设置超时
            tokio::select! {
                result = server_handle => {
                    match result {
                        Ok(_) => log::info!("Test HTTP server stopped gracefully"),
                        Err(e) => log::warn!("Test HTTP server stopped with error: {:?}", e),
                    }
                }
                _ = sleep(Duration::from_secs(5)) => {
                    log::warn!("Test HTTP server shutdown timeout");
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)] // 测试框架方法，用于清理测试数据
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 清理测试数据
        let mut tx = self.db.pool.begin().await?;

        sqlx::query("DELETE FROM account WHERE id > 1")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM role WHERE id > 2")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM organization WHERE id > 2")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM operation_log")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // 确保服务器在环境销毁时关闭
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.shutdown();
        }
    }
}

// 测试辅助函数
#[allow(dead_code)] // 测试框架辅助函数
pub async fn wait_for_server_ready(
    base_url: &str,
    max_attempts: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    for _ in 0..max_attempts {
        if let Ok(response) = client.get(format!("{}/api/health", base_url)).send().await {
            if response.status().is_success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    Err("Server did not become ready in time".into())
}

#[allow(dead_code)] // 测试框架辅助函数
pub fn assert_api_success(response: &Value) {
    assert_eq!(response["code"].as_i64().unwrap(), 200);
    assert!(response["data"].is_object() || response["data"].is_array());
}

#[allow(dead_code)] // 测试框架辅助函数
pub fn assert_api_error(response: &Value, expected_code: i64) {
    assert_eq!(response["code"].as_i64().unwrap(), expected_code);
    assert!(response["message"].is_string());
}
