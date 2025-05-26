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
            Account, AccountData, CreateAccountRequest, CurrentAccountResponseData,
            GetAccountRequest, GetAccountResponseData, ListAccountRequest, ListAccountResponseData,
        },
        auth::LoginRequest,
        menu::MenuTree,
        operation_log::{
            CreateOperationLogRequest, ListOperationLogRequest, ListOperationLogResponseData,
        },
        organization::{
            CreateOrganizationRequest, GetOrganizationRequest, GetOrganizationResponseData,
            OrganizationLimitType, OrganizationTree, children_organization_tree,
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
    async fn list_operation_log(
        &self,
        req: &ListOperationLogRequest,
    ) -> Result<ListOperationLogResponseData, Error> {
        let account_ids = self
            .repo
            .list_self_and_sub_ogranization_account_ids(
                req.current_user_id,
                OrganizationLimitType::SubOrganization,
            )
            .await?;
        let operation_logs = self
            .repo
            .list_operation_log(&req.page_filter, &account_ids)
            .await?;
        let total = self.repo.list_operation_log_count(&account_ids).await?;
        Ok(ListOperationLogResponseData::new(total, operation_logs))
    }
    async fn create_operation_log(&self, req: &CreateOperationLogRequest) -> Result<(), Error> {
        self.repo.create_operation_log(req).await?;
        Ok(())
    }
    async fn get_organization(
        &self,
        req: &GetOrganizationRequest,
    ) -> Result<GetOrganizationResponseData, Error> {
        self.repo
            .check_organization_user_creation_permission(
                req.current_user_id,
                req.id,
                OrganizationLimitType::SubOrganization,
            )
            .await?;
        let organization = self.repo.get_organization_by_id(req.id).await?;
        Ok(GetOrganizationResponseData { organization })
    }
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

        // 如果是超级管理员，返回空的组织限制列表，表示可以访问所有组织
        let organization_first_level = if is_admin {
            Vec::new() // 空列表表示无限制
        } else {
            self.repo
                .list_origanization_by_id(
                    current_account.organization_id,
                    is_admin,
                    OrganizationLimitType::FirstLevel,
                    organizations.clone(),
                )
                .await?
        };
        // 如果是超级管理员，获取所有组织ID用于授权检查
        let organization_sub_include = if is_admin {
            organizations.iter().map(|org| org.id).collect()
        } else {
            self.repo
                .list_origanization_by_id(
                    current_account.organization_id,
                    is_admin,
                    OrganizationLimitType::SubOrganizationIncludeSelf,
                    organizations,
                )
                .await?
        };
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
                let mut a = AccountData::new(account);
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

        Ok(children_organization_tree(
            &organizations,
            account.organization_id,
        ))
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
