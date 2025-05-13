use error_stack::{Result, ResultExt};
use std::collections::HashMap;

use crate::{
    domain::{
        models::{
            menu::{MenuTree, children_menu_tree},
            organization::CreateOrganizationRequest,
            page_utils::PageFilter,
            role::{CreateRoleRequest, ListRoleResponseData, RoleName},
        },
        ports::SysRepository,
    },
    errors::Error,
};

use super::db::Db;

impl SysRepository for Db {
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

    async fn create_role(&self, req: &CreateRoleRequest) -> Result<i64, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        if let Some(_) = self
            .fetch_role_by_name(&mut tx, req.name.as_ref())
            .await
            .change_context_lazy(|| Error::Message("failed to fetch role".to_string()))?
        {
            return Err(Error::BadRequest("role already exists".to_string()).into());
        }
        let id = self
            .save_role(&mut tx, req)
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
