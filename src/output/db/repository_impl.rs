use error_stack::{Result, ResultExt};
use std::collections::HashMap;

use crate::{
    domain::{
        models::{
            account::{Account, CreateAccountRequest},
            auth::LoginRequest,
            menu::{MenuTree, children_menu_tree},
            organization::{CreateOrganizationRequest, Organization, OrganizationLimitType},
            page_utils::PageFilter,
            role::{CreateRoleRequest, ListRoleResponseData, RoleName},
        },
        ports::SysRepository,
    },
    errors::Error,
    utils::password_hash::verify_password_hash,
};

use super::db::Db;

impl SysRepository for Db {
    async fn check_role_menu_subset(
        &self,
        assigner_user_id: i64,
        assignee_role_id: i64,
    ) -> Result<(), Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let assigner_account = self
            .fetch_account_by_id(&mut tx, assigner_user_id)
            .await
            .change_context_lazy(|| Error::Message("failed to filter account".to_string()))?;
        let assigner_role_menus = self
            .filter_role_menu_by_role_id(&mut tx, assigner_account.role_id)
            .await
            .change_context_lazy(|| Error::Message("failed to filter role menu".to_string()))?;
        let assignee_role_menus = self
            .filter_role_menu_by_role_id(&mut tx, assignee_role_id)
            .await
            .change_context_lazy(|| Error::Message("failed to filter role menu".to_string()))?;
        if !assigner_role_menus
            .iter()
            .all(|menu| assignee_role_menus.contains(menu))
        {
            return Err(Error::BadRequest(
                "assigner role menu not subset of assignee role menu".to_string(),
            )
            .into());
        }
        Ok(())
    }

    async fn login(&self, req: &LoginRequest) -> Result<Account, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let account = self.filter_account_by_name(&mut tx, &req.username).await?;
        if let Some(account) = account {
            if verify_password_hash(&req.password, &account.password) {
                return Ok(account);
            } else {
                log::error!("invalid account or password: {}", req.username);
            }
        }
        return Err(Error::BadRequest("invalid account or password".to_string()).into());
    }

    async fn check_organization_user_creation_permission(
        &self,
        current_user_id: i64,
        target_organization_id: i64,
        limit_type: OrganizationLimitType,
    ) -> Result<(), Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let account = self.fetch_account_by_id(&mut tx, current_user_id).await?;
        if account.organization_id == -1 && limit_type == OrganizationLimitType::FirstLevel {
            return Ok(());
        }
        let is_admin = account.id == 1;
        let organizations = self.all_organizations(&mut tx).await?;
        let organization_ids = self
            .list_origanization_by_id(target_organization_id, is_admin, limit_type, organizations)
            .await?;
        if organization_ids.contains(&target_organization_id) {
            return Ok(());
        }
        return Err(Error::BadRequest("no permission".to_string()).into());
    }

    async fn list_origanization_by_id(
        &self,
        target_organization_id: i64,
        is_admin: bool,
        limit_type: OrganizationLimitType,
        organizations: Vec<Organization>,
    ) -> Result<Vec<i64>, Error> {
        // admin 返回所有的组织ID
        if is_admin {
            let mut ids = organizations.iter().map(|o| o.id).collect::<Vec<i64>>();
            ids.push(-1);
            return Ok(ids);
        }

        // 非admin 但是根组织用户，返回所有一级组织及组织id
        if target_organization_id == -1 {
            return Ok(organizations.iter().map(|o| o.id).collect::<Vec<i64>>());
        }

        let mut organization_map = HashMap::new();
        let mut parent_id_map = HashMap::new();
        for o in organizations {
            parent_id_map.insert(o.id, o.parent_id);
            organization_map
                .entry(o.parent_id)
                .or_insert_with(Vec::new)
                .push(o.id);
        }

        match limit_type {
            OrganizationLimitType::FirstLevel => {
                let first_level_id = get_first_level_id(&parent_id_map, target_organization_id);
                return Ok(list_organization_by_user_contain(
                    first_level_id,
                    &organization_map,
                ));
            }
            OrganizationLimitType::SubOrganization => {
                return Ok(list_organization_by_user(
                    target_organization_id,
                    &organization_map,
                ));
            }
            OrganizationLimitType::SubOrganizationIncludeSelf => {
                return Ok(list_organization_by_user_contain(
                    target_organization_id,
                    &organization_map,
                ));
            }
            _ => {
                return Err(Error::BadRequest("invalid limit type".to_string()).into());
            }
        }
    }

    async fn create_account(&self, req: &CreateAccountRequest) -> Result<i64, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        if self
            .filter_account_by_name(&mut tx, &req.name)
            .await?
            .is_some()
        {
            return Err(Error::BadRequest("account already exists".to_string()).into());
        }
        let id = self.save_account(&mut tx, req).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(id)
    }

    async fn create_organization(&self, req: &CreateOrganizationRequest) -> Result<i64, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;

        if self
            .fetch_organization_by_name(&mut tx, &req.name)
            .await?
            .is_some()
        {
            return Err(Error::BadRequest("organization already exists".to_string()).into());
        }

        let id = self
            .save_organization(&mut tx, req)
            .await
            .change_context_lazy(|| Error::Message("failed to create organization".to_string()))?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(id)
    }

    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let menus = self
            .list_menu(&mut tx)
            .await
            .change_context_lazy(|| Error::Message("failed to list menu".to_string()))?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;

        let sid_map = HashMap::new();

        let menu_trees = children_menu_tree(&menus, &sid_map, -1);

        Ok(menu_trees)
    }

    async fn create_role(
        &self,
        req: &CreateRoleRequest,
        current_user_id: i64,
    ) -> Result<i64, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        if let Some(_) = self
            .filter_role_by_name(&mut tx, req.name.as_ref())
            .await
            .change_context_lazy(|| Error::Message("failed to fetch role".to_string()))?
        {
            return Err(Error::BadRequest("role already exists".to_string()).into());
        }
        let current_account = self
            .fetch_account_by_id(&mut tx, current_user_id)
            .await
            .change_context_lazy(|| {
                Error::Message("failed to fetch current account".to_string())
            })?;
        let id = self
            .save_role(&mut tx, req, current_user_id, &current_account.name)
            .await
            .change_context_lazy(|| Error::Message("failed to create role".to_string()))?;

        // 批量保存角色菜单关系
        self.save_role_menus(&mut tx, id, req.name.as_ref(), req.menus.as_ref())
            .await
            .change_context_lazy(|| Error::Message("failed to save role menus".to_string()))?;

        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(id)
    }

    async fn list_role(
        &self,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> Result<ListRoleResponseData, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let roles = self
            .filter_role(&mut tx, name, page_filter)
            .await
            .change_context_lazy(|| Error::Message("failed to list role".to_string()))?;

        let total = self
            .filter_role_count(&mut tx, name)
            .await
            .change_context_lazy(|| Error::Message("failed to filter role count".to_string()))?;

        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(ListRoleResponseData::new(total, roles))
    }
}

fn get_first_level_id(id_map: &HashMap<i64, i64>, id: i64) -> i64 {
    if let Some(v) = id_map.get(&id) {
        if *v == -1 {
            return id;
        }
        return get_first_level_id(id_map, *v);
    }
    return get_first_level_id(id_map, *id_map.get(&id).unwrap());
}

fn list_organization_by_user_contain(
    id: i64,
    organization_map: &HashMap<i64, Vec<i64>>,
) -> Vec<i64> {
    let mut ids = Vec::new();
    ids.push(id);
    if let Some(v) = organization_map.get(&id) {
        for child_id in v {
            ids.extend(list_organization_by_user_contain(
                *child_id,
                organization_map,
            ));
        }
    }
    ids
}

fn list_organization_by_user(id: i64, organization_map: &HashMap<i64, Vec<i64>>) -> Vec<i64> {
    let mut ids = Vec::new();
    if let Some(v) = organization_map.get(&id) {
        for child_id in v {
            ids.extend(list_organization_by_user(*child_id, organization_map));
        }
    }
    ids
}
