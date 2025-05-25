use crate::{
    domain::{
        models::organization::{all_tree, first_level_tree},
        ports::SysRepository,
    },
    errors::Error,
};

use super::{
    models::{
        account::{
            Account, AcountData, CreateAccountRequest, CurrentAccountResponseData,
            GetAccountRequest, GetAccountResponseData, ListAccountRequest, ListAccountResponseData,
        },
        auth::LoginRequest,
        menu::MenuTree,
        organization::{
            CreateOrganizationRequest, OrganizationLimitType, OrganizationTree,
            children_organization_tree,
        },
        page_utils::PageFilter,
        role::{
            CreateRoleRequest, GetRoleRequest, GetRoleResponseData, ListRoleResponseData, RoleName,
        },
        route::{RouteMethod, RoutePath},
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
    async fn get_account(&self, req: &GetAccountRequest) -> Result<GetAccountResponseData, Error> {
        let account = self.repo.get_account_by_id(req.id).await?;
        self.repo
            .check_organization_user_creation_permission(
                req.current_user_id,
                account.organization_id,
                OrganizationLimitType::FirstLevel,
            )
            .await?;
        let menus = self.repo.list_menu_by_role_id(account.role_id).await?;
        Ok(GetAccountResponseData::new(account, menus))
    }
    async fn list_account(
        &self,
        req: &ListAccountRequest,
    ) -> Result<ListAccountResponseData, Error> {
        let current_account = self.repo.get_account_by_id(req.current_user_id).await?;
        let is_admin = current_account.id == 1;
        let organizations = self.repo.all_organizations().await?;
        let organization_first_level = self
            .repo
            .list_origanization_by_id(
                current_account.organization_id,
                is_admin,
                OrganizationLimitType::FirstLevel,
                organizations.clone(),
            )
            .await?;
        let organization_sub_include = self
            .repo
            .list_origanization_by_id(
                current_account.organization_id,
                is_admin,
                OrganizationLimitType::SubOrganizationIncludeSelf,
                organizations,
            )
            .await?;
        let account_list = self
            .repo
            .list_account(
                req.account_name.as_ref(),
                req.organization_id,
                &organization_first_level,
                &req.page_filter,
            )
            .await?;
        let total = self
            .repo
            .count_account(
                req.account_name.as_ref(),
                req.organization_id,
                &organization_first_level,
            )
            .await?;
        let account_data_list = account_list
            .iter()
            .map(|account| {
                let mut a = AcountData::new(&account);
                if organization_sub_include.contains(&account.organization_id)
                    || account.id == current_account.id
                {
                    a.is_authorized = true;
                }
                a
            })
            .collect();
        Ok(ListAccountResponseData::new(total, account_data_list))
    }
    async fn check_permission(
        &self,
        user_id: i64,
        path: &RoutePath,
        method: &RouteMethod,
    ) -> Result<bool, Error> {
        let res = self.repo.check_permission(user_id, path, method).await?;
        Ok(res)
    }

    async fn organization_tree(
        &self,
        current_user_id: i64,
        limit_type: OrganizationLimitType,
    ) -> Result<Vec<OrganizationTree>, Error> {
        let organizations = self.repo.all_organizations().await?;
        let account = self.repo.get_account_by_id(current_user_id).await?;
        if limit_type == OrganizationLimitType::Root {
            return Ok(vec![first_level_tree(
                &organizations,
                account.organization_id,
                account.organization_id,
            )]);
        }
        if account.organization_id == -1 {
            return Ok(all_tree(&organizations));
        }
        if limit_type == OrganizationLimitType::FirstLevel {
            return Ok(vec![first_level_tree(
                &organizations,
                account.organization_id,
                account.organization_id,
            )]);
        }

        return Ok(children_organization_tree(
            &organizations,
            account.organization_id,
        ));
    }

    async fn current_account(
        &self,
        current_user_id: i64,
    ) -> Result<CurrentAccountResponseData, Error> {
        let account = self.repo.get_account_by_id(current_user_id).await?;
        self.repo
            .check_organization_user_creation_permission(
                current_user_id,
                account.organization_id,
                OrganizationLimitType::FirstLevel,
            )
            .await?;
        let menus = self.repo.list_menu_by_role_id(account.role_id).await?;
        Ok(CurrentAccountResponseData::new(account, menus))
    }
    async fn get_role(&self, req: &GetRoleRequest) -> Result<GetRoleResponseData, Error> {
        let role = self.repo.get_role_by_id(req.id).await?;
        let menus = self.repo.list_menu_by_role_id(req.id).await?;
        Ok(GetRoleResponseData::new(role, menus))
    }

    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let res = self.repo.list_menu().await?;
        Ok(res)
    }

    async fn create_role(
        &self,
        req: &CreateRoleRequest,
        current_user_id: i64,
    ) -> Result<i64, Error> {
        let res = self.repo.create_role(req, current_user_id).await?;
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
