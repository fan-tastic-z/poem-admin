use error_stack::{Result, ResultExt};
use std::collections::HashMap;

use crate::{
    domain::{
        models::{
            menu::{MenuTree, children_menu_tree},
            role::CreateRoleRequest,
        },
        ports::SysRepository,
    },
    errors::Error,
};

use super::db::Db;

impl SysRepository for Db {
    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .change_context_lazy(|| Error("failed to begin transaction".to_string()))?;
        let menus = self
            .list_menu(&mut tx)
            .await
            .change_context_lazy(|| Error("failed to list menu".to_string()))?;
        tx.commit()
            .await
            .change_context_lazy(|| Error("failed to commit transaction".to_string()))?;

        let sid_map = HashMap::new();

        let menu_trees = children_menu_tree(&menus, &sid_map, -1);

        Ok(menu_trees)
    }

    async fn create_role(&self, req: &CreateRoleRequest) -> Result<i64, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .change_context_lazy(|| Error("failed to begin transaction".to_string()))?;
        if let Some(_) = self
            .fetch_role_by_name(&mut tx, req.name.as_ref())
            .await
            .change_context_lazy(|| Error("failed to fetch role".to_string()))?
        {
            return Err(Error("role already exists".to_string()).into());
        }
        let id = self
            .save_role(&mut tx, req)
            .await
            .change_context_lazy(|| Error("failed to create role".to_string()))?;

        tx.commit()
            .await
            .change_context_lazy(|| Error("failed to commit transaction".to_string()))?;
        Ok(id)
    }
}
