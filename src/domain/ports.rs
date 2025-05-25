use crate::errors::Error;
use std::future::Future;

use super::models::{
    account::{
        Account, AccountName, CreateAccountRequest, CurrentAccountResponseData, GetAccountRequest,
        GetAccountResponseData, ListAccountRequest, ListAccountResponseData,
    },
    auth::LoginRequest,
    menu::MenuTree,
    organization::{
        CreateOrganizationRequest, GetOrganizationRequest, GetOrganizationResponseData,
        Organization, OrganizationLimitType, OrganizationTree,
    },
    page_utils::PageFilter,
    role::{
        CreateRoleRequest, GetRoleRequest, GetRoleResponseData, ListRoleResponseData, Role,
        RoleName,
    },
    route::{RouteMethod, RoutePath},
};
use error_stack::Result;

pub trait SysService: Clone + Send + Sync + 'static {
    fn list_account(
        &self,
        req: &ListAccountRequest,
    ) -> impl Future<Output = Result<ListAccountResponseData, Error>> + Send;

    fn organization_tree(
        &self,
        current_user_id: i64,
        limit_type: OrganizationLimitType,
    ) -> impl Future<Output = Result<Vec<OrganizationTree>, Error>> + Send;
    fn current_account(
        &self,
        current_user_id: i64,
    ) -> impl Future<Output = Result<CurrentAccountResponseData, Error>> + Send;
    fn list_menu(&self) -> impl Future<Output = Result<Vec<MenuTree>, Error>> + Send;
    fn create_role(
        &self,
        req: &CreateRoleRequest,
        current_user_id: i64,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
    fn list_role(
        &self,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> impl Future<Output = Result<ListRoleResponseData, Error>> + Send;

    fn create_organization(
        &self,
        req: &CreateOrganizationRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn create_account(
        &self,
        req: &CreateAccountRequest,
        current_user_id: i64,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<Account, Error>> + Send;

    fn get_role(
        &self,
        req: &GetRoleRequest,
    ) -> impl Future<Output = Result<GetRoleResponseData, Error>> + Send;
    fn check_permission(
        &self,
        user_id: i64,
        path: &RoutePath,
        method: &RouteMethod,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_account(
        &self,
        req: &GetAccountRequest,
    ) -> impl Future<Output = Result<GetAccountResponseData, Error>> + Send;

    fn get_organization(
        &self,
        req: &GetOrganizationRequest,
    ) -> impl Future<Output = Result<GetOrganizationResponseData, Error>> + Send;
}

pub trait SysRepository: Clone + Send + Sync + 'static {
    fn check_permission(
        &self,
        user_id: i64,
        path: &RoutePath,
        method: &RouteMethod,
    ) -> impl Future<Output = Result<bool, Error>> + Send;
    fn all_organizations(&self) -> impl Future<Output = Result<Vec<Organization>, Error>> + Send;
    fn get_account_by_id(&self, id: i64) -> impl Future<Output = Result<Account, Error>> + Send;

    fn get_organization_by_id(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Organization, Error>> + Send;

    fn list_menu_by_role_id(
        &self,
        role_id: i64,
    ) -> impl Future<Output = Result<Vec<MenuTree>, Error>> + Send;

    fn list_menu(&self) -> impl Future<Output = Result<Vec<MenuTree>, Error>> + Send;
    fn create_role(
        &self,
        req: &CreateRoleRequest,
        current_user_id: i64,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
    fn list_role(
        &self,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> impl Future<Output = Result<ListRoleResponseData, Error>> + Send;

    fn create_organization(
        &self,
        req: &CreateOrganizationRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn create_account(
        &self,
        req: &CreateAccountRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn list_origanization_by_id(
        &self,
        id: i64,
        is_admin: bool,
        limit_type: OrganizationLimitType,
        organizations: Vec<Organization>,
    ) -> impl Future<Output = Result<Vec<i64>, Error>> + Send;

    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<Account, Error>> + Send;

    fn check_organization_user_creation_permission(
        &self,
        current_user_id: i64,
        target_organization_id: i64,
        limit_type: OrganizationLimitType,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn check_role_menu_subset(
        &self,
        assigner_user_id: i64,
        assignee_role_id: i64,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn get_role_by_id(&self, id: i64) -> impl Future<Output = Result<Role, Error>> + Send;

    fn list_account(
        &self,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
        page_filter: &PageFilter,
    ) -> impl Future<Output = Result<Vec<Account>, Error>> + Send;

    fn count_account(
        &self,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
    ) -> impl Future<Output = Result<i64, Error>> + Send;
}
