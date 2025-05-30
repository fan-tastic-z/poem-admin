mod common;

use common::TestEnvironment;
use poem_admin::domain::{
    models::{
        account::{AccountName, AccountPassword, CreateAccountRequest, GetAccountRequest},
        auth::LoginRequest,
        operation_log::{
            CreateOperationLogRequest, ListOperationLogRequest, OperationLogDescription,
            OperationLogIpAddress, OperationLogModule, OperationLogUserAgent, OperationResult,
            OperationType,
        },
        organization::{CreateOrganizationRequest, GetOrganizationRequest, OrganizationName},
        page_utils::{PageFilter, PageNo, PageSize},
        role::{CreateRoleRequest, GetRoleRequest, RoleName},
    },
    ports::SysService,
};

#[tokio::test]
async fn test_authentication_flow() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 测试成功登录
    let login_request = LoginRequest {
        username: AccountName::try_new("admin".to_string()).expect("Valid username"),
        password: AccountPassword::try_new("admin123".to_string()).expect("Valid password"),
    };

    let account = env
        .ctx
        .sys_service
        .login(&login_request)
        .await
        .expect("Login should succeed");
    assert_eq!(account.name.as_str(), "admin");
    assert_eq!(account.organization_id, 1);

    // 测试登录失败
    let invalid_login = LoginRequest {
        username: AccountName::try_new("admin".to_string()).expect("Valid username"),
        password: AccountPassword::try_new("wrong_password".to_string()).expect("Valid password"),
    };

    let result = env.ctx.sys_service.login(&invalid_login).await;
    assert!(result.is_err(), "Login with wrong password should fail");

    // 测试 JWT Token 生成
    let token = env.get_admin_token().await.expect("Should generate token");
    assert!(!token.is_empty(), "Token should not be empty");

    println!("✓ Authentication flow test passed");
}

#[tokio::test]
async fn test_account_management() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 创建测试用户
    let (username, password, user_id) = env
        .create_test_user()
        .await
        .expect("Should create test user");

    // 验证用户创建成功
    let get_request = GetAccountRequest::new(user_id, 1); // user_id, current_user_id
    let account_detail = env
        .ctx
        .sys_service
        .get_account(&get_request)
        .await
        .expect("Should get created account");
    assert_eq!(account_detail.account.name.as_str(), &username);
    assert_eq!(account_detail.account.role_id, 2); // 普通用户角色

    // 测试用户登录
    let login_request = LoginRequest {
        username: AccountName::try_new(username.clone()).expect("Valid username"),
        password: AccountPassword::try_new(password).expect("Valid password"),
    };
    let login_result = env
        .ctx
        .sys_service
        .login(&login_request)
        .await
        .expect("Test user should be able to login");
    assert_eq!(login_result.id, user_id);

    println!("✓ Account management test passed");
}

#[tokio::test]
async fn test_role_management() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 创建新角色
    let role_request = CreateRoleRequest::new(
        RoleName::try_new("test_role".to_string()).expect("Valid role name"),
        Some(
            poem_admin::domain::models::role::RoleDescription::try_new("测试角色描述".to_string())
                .expect("Valid description"),
        ),
        true,   // is_deletable
        vec![], // menu_requests
    );

    let role_id = env
        .ctx
        .sys_service
        .create_role(&role_request, 1)
        .await
        .expect("Should create role");

    // 验证角色创建成功
    let get_role_request = GetRoleRequest::new(role_id);
    let created_role = env
        .ctx
        .sys_service
        .get_role(&get_role_request)
        .await
        .expect("Should get created role");
    assert_eq!(created_role.role.name.as_str(), "test_role");

    // 测试角色列表查询
    let page_filter = PageFilter::new(
        PageNo::try_new(1).expect("Valid page no"),
        PageSize::try_new(10).expect("Valid page size"),
    );
    let roles = env
        .ctx
        .sys_service
        .list_role(None, &page_filter)
        .await
        .expect("Should list roles");

    assert!(roles.total >= 3); // 至少有2个初始角色 + 1个新创建的角色
    let role_names: Vec<&str> = roles.data.iter().map(|r| r.name.as_str()).collect();
    assert!(role_names.contains(&"test_role"));

    println!("✓ Role management test passed");
}

#[tokio::test]
async fn test_organization_management() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 测试获取组织树
    let organizations = env
        .ctx
        .sys_service
        .organization_tree(
            1,
            poem_admin::domain::models::organization::OrganizationLimitType::FirstLevel,
        )
        .await
        .expect("Should get organization tree");
    assert!(!organizations.is_empty()); // 应该有组织数据

    // 创建新组织
    let org_request = CreateOrganizationRequest::new(
        OrganizationName::try_new("测试部门".to_string()).expect("Valid org name"),
        1, // parent_id
        Some(OrganizationName::try_new("总公司".to_string()).expect("Valid parent name")),
    );

    let org_id = env
        .ctx
        .sys_service
        .create_organization(org_request)
        .await
        .expect("Should create organization");

    // 验证组织创建成功
    let get_org_request = GetOrganizationRequest::new(org_id, 1);
    let created_org = env
        .ctx
        .sys_service
        .get_organization(&get_org_request)
        .await
        .expect("Should get created organization");
    assert_eq!(created_org.organization.name.as_str(), "测试部门");
    assert_eq!(created_org.organization.parent_id, 1);

    println!("✓ Organization management test passed");
}

#[tokio::test]
async fn test_menu_and_permissions() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 测试获取菜单
    let menus = env
        .ctx
        .sys_service
        .list_menu()
        .await
        .expect("Should get menus");
    assert!(!menus.is_empty(), "Should have menu data");

    // 验证菜单结构
    let menu_names: Vec<&str> = menus.iter().map(|m| m.name.as_str()).collect();
    assert!(menu_names.contains(&"用户管理"));

    println!("✓ Menu and permissions test passed");
}

#[tokio::test]
async fn test_operation_log() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 创建操作日志
    let log_request = CreateOperationLogRequest::builder()
        .account_id(1)
        .account_name(AccountName::try_new("admin".to_string()).expect("Valid account name"))
        .ip_address(OperationLogIpAddress::try_new("127.0.0.1".to_string()).expect("Valid IP"))
        .user_agent(OperationLogUserAgent::new("test-agent".to_string()))
        .operation_type(OperationType::Create)
        .operation_module(OperationLogModule::new("account".to_string()))
        .operation_description(OperationLogDescription::new("创建用户测试".to_string()))
        .operation_result(OperationResult::Success)
        .build()
        .expect("Should build operation log request");

    // 记录操作日志
    env.ctx
        .sys_service
        .create_operation_log(&log_request)
        .await
        .expect("Should create operation log");

    // 查询操作日志
    let list_request = ListOperationLogRequest {
        page_filter: PageFilter::new(
            PageNo::try_new(1).expect("Valid page no"),
            PageSize::try_new(10).expect("Valid page size"),
        ),
        current_user_id: 1,
    };
    let logs = env
        .ctx
        .sys_service
        .list_operation_log(&list_request)
        .await
        .expect("Should list operation logs");

    assert!(logs.total > 0, "Should have operation logs");
    assert!(
        !logs.operation_logs.is_empty(),
        "Should have operation log data"
    );

    // 验证日志内容
    let log_descriptions: Vec<&str> = logs
        .operation_logs
        .iter()
        .map(|l| l.operation_description.as_ref())
        .collect();
    assert!(log_descriptions.contains(&"创建用户测试"));

    println!("✓ Operation log test passed");
}

#[tokio::test]
async fn test_error_handling() {
    let env = TestEnvironment::new()
        .await
        .expect("Failed to setup test environment");

    // 测试重复用户名创建
    let duplicate_request = CreateAccountRequest::new(
        AccountName::try_new("admin".to_string()).expect("Valid username"),
        AccountPassword::try_new(
            poem_admin::utils::password_hash::compute_password_hash("password123")
                .expect("Hash password"),
        )
        .expect("Valid password"),
        1,
        OrganizationName::try_new("总公司".to_string()).expect("Valid org name"),
        2,
        RoleName::try_new("普通用户".to_string()).expect("Valid role name"),
    );

    let result = env
        .ctx
        .sys_service
        .create_account(duplicate_request, 1)
        .await;
    assert!(result.is_err(), "Should not allow duplicate usernames");

    // 测试无效用户ID查询
    let invalid_get_request = GetAccountRequest::new(99999, 1);
    let result = env.ctx.sys_service.get_account(&invalid_get_request).await;
    assert!(result.is_err(), "Should return error for non-existent user");

    // 测试无效角色ID查询
    let invalid_role_request = GetRoleRequest::new(99999);
    let result = env.ctx.sys_service.get_role(&invalid_role_request).await;
    assert!(result.is_err(), "Should return error for non-existent role");

    println!("✓ Error handling test passed");
}
