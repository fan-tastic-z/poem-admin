use error_stack::{Result, ResultExt};
use sqlx::Row;
use std::collections::HashMap;

use crate::{
    domain::{
        models::menu::{Menu, MenuName, MenuTree, children_menu_tree},
        ports::SysRepository,
    },
    errors::Error,
};

use super::db::Db;

impl SysRepository for Db {
    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let rows = self
            .list_menu(&mut tx)
            .await
            .change_context_lazy(|| Error::Message("failed to list menu".to_string()))?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;

        // Convert rows to menus
        let menus: Result<Vec<Menu>, Error> =
            rows.iter()
                .map(|row| {
                    let name = MenuName::try_new(row.get::<&str, _>("name")).change_context_lazy(
                        || Error::Message("failed to get menu name".to_string()),
                    )?;

                    let parent_name = row.get::<Option<&str>, _>("parent_name")
                        .and_then(|name| if name.is_empty() { None } else { Some(name) })
                        .map(|name| MenuName::try_new(name))
                        .transpose()
                        .change_context_lazy(|| Error::Message("failed to get parent menu name".to_string()))?;

                    Ok(Menu::new(
                        row.get("id"),
                        name,
                        row.get("parent_id"),
                        parent_name,
                    ))
                })
                .collect();

        // Create MenuTrees from the menus
        let menus = menus?;
        let sid_map = HashMap::new();

        let menu_trees = children_menu_tree(&menus, &sid_map, -1);

        Ok(menu_trees)
    }
}
