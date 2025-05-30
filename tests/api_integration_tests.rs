mod common;

use common::TestEnvironment;
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

// 注意：这些测试需要实际启动 HTTP 服务器
// 由于项目结构的复杂性，这里提供测试框架，实际运行时可能需要调整

/// 测试健康检查端点
#[tokio::test]
async fn test_health_endpoint() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 启动HTTP服务器
    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();
    let response = client
        .get(format!("{}/api/health", env.base_url))
        .send()
        .await
        .expect("Health check should work");

    assert_eq!(response.status(), StatusCode::OK);

    let body_text = response.text().await.expect("Should get response text");
    assert_eq!(body_text, "OK");

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Health endpoint test passed");
}

/// 测试登录API
#[tokio::test]
async fn test_login_api() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 测试成功登录
    let login_response = client
        .post(format!("{}/api/login", env.base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .expect("Login request should work");

    assert_eq!(login_response.status(), StatusCode::OK);

    let login_data: Value = login_response
        .json()
        .await
        .expect("Should parse login response");

    // 验证响应结构
    assert!(login_data["status_code"].as_u64().unwrap() == 200);
    let data = &login_data["data"];

    let token = data["token"].as_str().expect("Should have token");
    let user_id = data["user_id"].as_i64().expect("Should have user_id");
    let expires_in = data["expires_in"].as_u64().expect("Should have expires_in");

    assert!(!token.is_empty(), "Token should not be empty");
    assert_eq!(user_id, 1, "Admin user ID should be 1");
    assert!(expires_in > 0, "Token should have expiration time");

    // 测试登录失败
    let invalid_login_response = client
        .post(format!("{}/api/login", env.base_url))
        .json(&json!({
            "username": "admin",
            "password": "wrong_password"
        }))
        .send()
        .await
        .expect("Invalid login request should work");

    assert!(invalid_login_response.status().is_client_error());

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Login API test passed");
}

/// 测试账户管理API
#[tokio::test]
async fn test_account_apis() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 获取管理员 token
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    // 测试获取当前用户信息
    let current_user_response = client
        .get(format!("{}/api/accounts/current", env.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Get current user should work");

    assert_eq!(current_user_response.status(), StatusCode::OK);

    let current_user_data: Value = current_user_response
        .json()
        .await
        .expect("Should parse current user response");

    assert_eq!(current_user_data["status_code"].as_u64().unwrap(), 200);
    assert_eq!(current_user_data["data"]["name"], "admin");
    assert!(
        current_user_data["data"]["menus"].is_array(),
        "Should have menus array"
    );

    // 测试创建账户
    let test_username = format!("test{}", chrono::Utc::now().timestamp() % 1000);
    let create_account_response = client
        .post(format!("{}/api/accounts", env.base_url))
        .bearer_auth(&token)
        .json(&json!({
            "name": test_username,
            "password": "test123456",
            "email": "test@example.com",
            "organization_id": 1,
            "organization_name": "总公司",
            "role_id": 2,
            "role_name": "普通用户"
        }))
        .send()
        .await
        .expect("Create account should work");

    let status = create_account_response.status();
    if status != StatusCode::CREATED {
        let error_text = create_account_response.text().await.unwrap();
        println!(
            "Create account failed with status: {}, body: {}",
            status, error_text
        );
        panic!("Create account failed");
    }

    let create_data: Value = create_account_response
        .json()
        .await
        .expect("Should parse create response");

    println!(
        "Create account response: {}",
        serde_json::to_string_pretty(&create_data).unwrap()
    );

    assert_eq!(create_data["status_code"].as_u64().unwrap(), 201);
    let new_user_id = create_data["data"]["id"]
        .as_i64()
        .expect("Should return new user ID");
    assert!(new_user_id > 0, "Should return valid user ID");

    // 测试获取账户列表
    let list_accounts_response = client
        .get(format!(
            "{}/api/accounts?page_no=1&page_size=10",
            env.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("List accounts should work");

    let status = list_accounts_response.status();
    if status != StatusCode::OK {
        let error_text = list_accounts_response.text().await.unwrap();
        println!(
            "List accounts failed with status: {}, body: {}",
            status, error_text
        );
        panic!("List accounts failed");
    }

    let list_data: Value = list_accounts_response
        .json()
        .await
        .expect("Should parse list response");

    assert_eq!(list_data["status_code"].as_u64().unwrap(), 200);
    assert!(
        list_data["data"]["total"].as_i64().unwrap() >= 2,
        "Should have at least 2 accounts (admin + created user)"
    );

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Account APIs test passed");
}

/// 测试角色管理API
#[tokio::test]
async fn test_role_apis() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 获取管理员 token
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    // 测试获取角色列表
    let list_roles_response = client
        .get(format!("{}/api/roles?page_no=1&page_size=10", env.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("List roles should work");

    assert_eq!(list_roles_response.status(), StatusCode::OK);

    let list_data: Value = list_roles_response
        .json()
        .await
        .expect("Should parse list response");

    assert_eq!(list_data["status_code"].as_u64().unwrap(), 200);
    assert!(
        list_data["data"]["total"].as_i64().unwrap() >= 2,
        "Should have at least 2 roles"
    );

    // 测试创建角色
    let test_role_name = format!("role{}", chrono::Utc::now().timestamp() % 1000);
    let create_role_response = client
        .post(format!("{}/api/roles", env.base_url))
        .bearer_auth(&token)
        .json(&json!({
            "name": test_role_name,
            "description": "Test role",
            "is_deletable": true,
            "menus": []
        }))
        .send()
        .await
        .expect("Create role should work");

    let status = create_role_response.status();
    if status != StatusCode::CREATED {
        let error_text = create_role_response.text().await.unwrap();
        println!(
            "Create role failed with status: {}, body: {}",
            status, error_text
        );
        panic!("Create role failed");
    }

    let create_data: Value = create_role_response
        .json()
        .await
        .expect("Should parse create response");

    assert_eq!(create_data["status_code"].as_u64().unwrap(), 201);
    let new_role_id = create_data["data"]["id"]
        .as_i64()
        .expect("Should return new role ID");
    assert!(new_role_id > 0, "Should return valid role ID");

    // 测试获取特定角色信息
    let get_role_response = client
        .get(format!("{}/api/roles/{}/detail", env.base_url, new_role_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Get role should work");

    assert_eq!(get_role_response.status(), StatusCode::OK);

    let role_data: Value = get_role_response
        .json()
        .await
        .expect("Should parse role response");

    assert_eq!(role_data["status_code"].as_u64().unwrap(), 200);
    assert_eq!(role_data["data"]["name"], test_role_name);

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Role APIs test passed");
}

/// 测试组织管理API
#[tokio::test]
async fn test_organization_apis() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 获取管理员 token
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    // 测试获取组织树
    let org_tree_response = client
        .get(format!(
            "{}/api/organizations/tree?limit_type=Root",
            env.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Get organization tree should work");

    let status = org_tree_response.status();
    if status != StatusCode::OK {
        let error_text = org_tree_response.text().await.unwrap();
        println!(
            "Get organization tree failed with status: {}, body: {}",
            status, error_text
        );
        panic!("Get organization tree failed");
    }

    let tree_data: Value = org_tree_response
        .json()
        .await
        .expect("Should parse tree response");

    assert_eq!(tree_data["status_code"].as_u64().unwrap(), 200);
    assert!(
        tree_data["data"]["organizations"].is_array(),
        "Should return array of organizations"
    );

    // 测试创建组织
    let test_org_name = format!("org{}", chrono::Utc::now().timestamp() % 1000);
    let create_org_response = client
        .post(format!("{}/api/organizations", env.base_url))
        .bearer_auth(&token)
        .json(&json!({
            "name": test_org_name,
            "parent_id": 1,
            "parent_name": "总公司"
        }))
        .send()
        .await
        .expect("Create organization should work");

    let status = create_org_response.status();
    if status != StatusCode::CREATED {
        let error_text = create_org_response.text().await.unwrap();
        println!(
            "Create organization failed with status: {}, body: {}",
            status, error_text
        );
        panic!("Create organization failed");
    }

    let create_data: Value = create_org_response
        .json()
        .await
        .expect("Should parse create response");

    assert_eq!(create_data["status_code"].as_u64().unwrap(), 201);
    let new_org_id = create_data["data"]["id"]
        .as_i64()
        .expect("Should return new organization ID");
    assert!(new_org_id > 0, "Should return valid organization ID");

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Organization APIs test passed");
}

/// 测试菜单API
#[tokio::test]
async fn test_menu_apis() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 获取管理员 token
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    // 测试获取菜单列表
    let menu_response = client
        .get(format!("{}/api/menus", env.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Get menus should work");

    assert_eq!(menu_response.status(), StatusCode::OK);

    let menu_data: Value = menu_response
        .json()
        .await
        .expect("Should parse menu response");

    assert_eq!(menu_data["status_code"].as_u64().unwrap(), 200);
    assert!(
        menu_data["data"]["menus"].is_array(),
        "Should return array of menus"
    );

    let menus = menu_data["data"]["menus"].as_array().unwrap();
    assert!(!menus.is_empty(), "Should have some menus");

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Menu APIs test passed");
}

/// 测试权限验证
#[tokio::test]
async fn test_authorization() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 测试无token访问受保护端点
    let unauthorized_response = client
        .get(format!("{}/api/accounts/current", env.base_url))
        .send()
        .await
        .expect("Unauthorized request should work");

    assert_eq!(unauthorized_response.status(), StatusCode::UNAUTHORIZED);

    // 测试无效token
    let invalid_token_response = client
        .get(format!("{}/api/accounts/current", env.base_url))
        .bearer_auth("invalid_token")
        .send()
        .await
        .expect("Invalid token request should work");

    assert_eq!(invalid_token_response.status(), StatusCode::UNAUTHORIZED);

    // 测试有效token访问
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    let authorized_response = client
        .get(format!("{}/api/accounts/current", env.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Authorized request should work");

    assert_eq!(authorized_response.status(), StatusCode::OK);

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Authorization test passed");
}

/// 测试操作日志记录
#[tokio::test]
async fn test_operation_log_recording() {
    let mut env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    let server_addr = env
        .start_http_server()
        .await
        .expect("Failed to start HTTP server");

    println!("🚀 Test server started on {}", server_addr);

    let client = Client::new();

    // 获取管理员 token
    let token = get_admin_token_via_api(&client, &env.base_url)
        .await
        .expect("Should get admin token");

    // 执行一些会产生操作日志的操作
    let test_username = format!("log{}", chrono::Utc::now().timestamp() % 1000);
    let _create_response = client
        .post(format!("{}/api/accounts", env.base_url))
        .bearer_auth(&token)
        .json(&json!({
            "name": test_username,
            "password": "test123456",
            "email": "logtest@example.com",
            "organization_id": 1,
            "organization_name": "总公司",
            "role_id": 2,
            "role_name": "普通用户"
        }))
        .send()
        .await
        .expect("Create account should work");

    // 等待操作日志记录
    sleep(Duration::from_millis(100)).await;

    // 查询操作日志
    let log_response = client
        .get(format!(
            "{}/api/operation-logs?page_no=1&page_size=10",
            env.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Get operation logs should work");

    assert_eq!(log_response.status(), StatusCode::OK);

    let log_data: Value = log_response
        .json()
        .await
        .expect("Should parse log response");

    assert_eq!(log_data["status_code"].as_u64().unwrap(), 200);
    assert!(
        log_data["data"]["total"].as_i64().unwrap() > 0,
        "Should have operation logs"
    );

    env.stop_http_server().await.expect("Failed to stop server");
    println!("✅ Operation log recording test passed");
}

/// 辅助函数：通过API获取管理员token
async fn get_admin_token_via_api(
    client: &Client,
    base_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .post(format!("{}/api/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let token = data["data"]["token"]
        .as_str()
        .ok_or("No token in response")?
        .to_string();

    Ok(token)
}

// 注意：实际的服务器启动函数需要根据项目结构实现
// async fn start_test_server(ctx: Ctx<Service<Db>>) -> tokio::task::JoinHandle<()> {
//     tokio::spawn(async move {
//         // 启动 HTTP 服务器的逻辑
//         // 这需要根据 poem-admin 的实际 HTTP 服务器启动代码来实现
//     })
// }
