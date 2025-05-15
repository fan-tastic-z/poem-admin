use crate::errors::Error;
use std::future::Future;

use super::models::{
    account::{Account, CreateAccountRequest},
    auth::LoginRequest,
    menu::MenuTree,
    organization::{CreateOrganizationRequest, OrganizationLimitType},
    page_utils::PageFilter,
    role::{CreateRoleRequest, ListRoleResponseData, RoleName},
};
use error_stack::Result;

pub trait SysService: Clone + Send + Sync + 'static {
    fn list_menu(&self) -> impl Future<Output = Result<Vec<MenuTree>, Error>> + Send;
    fn create_role(
        &self,
        req: &CreateRoleRequest,
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

    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<Account, Error>> + Send;
}

pub trait SysRepository: Clone + Send + Sync + 'static {
    fn list_menu(&self) -> impl Future<Output = Result<Vec<MenuTree>, Error>> + Send;
    fn create_role(
        &self,
        req: &CreateRoleRequest,
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
    ) -> impl Future<Output = Result<Vec<i64>, Error>> + Send;

    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<Account, Error>> + Send;
}
