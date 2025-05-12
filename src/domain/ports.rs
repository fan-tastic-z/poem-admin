use crate::errors::Error;
use std::future::Future;

use super::models::{
    menu::MenuTree,
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
}
