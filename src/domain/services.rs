use crate::{domain::ports::SysRepository, errors::Error};

use super::{
    models::{
        account::{Account, CreateAccountRequest},
        auth::LoginRequest,
        menu::MenuTree,
        organization::{CreateOrganizationRequest, OrganizationLimitType},
        page_utils::PageFilter,
        role::{CreateRoleRequest, ListRoleResponseData, RoleName},
    },
    ports::SysService,
};
use error_stack::Result;

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: SysRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: SysRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> SysService for Service<R>
where
    R: SysRepository,
{
    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let res = self.repo.list_menu().await?;
        Ok(res)
    }

    async fn create_role(&self, req: &CreateRoleRequest) -> Result<i64, Error> {
        let res = self.repo.create_role(req).await?;
        Ok(res)
    }

    async fn list_role(
        &self,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> Result<ListRoleResponseData, Error> {
        let res = self.repo.list_role(name, page_filter).await?;
        Ok(res)
    }

    async fn create_organization(&self, req: &CreateOrganizationRequest) -> Result<i64, Error> {
        let res = self.repo.create_organization(req).await?;
        Ok(res)
    }

    async fn create_account(
        &self,
        req: &CreateAccountRequest,
        current_user_id: i64,
    ) -> Result<i64, Error> {
        self.repo
            .check_organization_user_creation_permission(
                current_user_id,
                req.organization_id,
                OrganizationLimitType::FirstLevel,
            )
            .await?;
        let res = self.repo.create_account(req).await?;
        Ok(res)
    }

    async fn login(&self, req: &LoginRequest) -> Result<Account, Error> {
        let res = self.repo.login(req).await?;
        Ok(res)
    }
}
