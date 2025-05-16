use sqlx::{Postgres, QueryBuilder, Transaction};

use crate::domain::models::{role::CreateRoleMenuRequest, role_menu::RoleMenu};

use super::db::Db;

impl Db {
    pub async fn save_role_menus(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        role_id: i64,
        role_name: &str,
        req: &[CreateRoleMenuRequest],
    ) -> Result<(), sqlx::Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
        INSERT INTO role_menu (role_id, role_name, menu_id, menu_name)
        "#,
        );
        query_builder.push_values(req, |mut b, role_menu| {
            b.push_bind(role_id)
                .push_bind(role_name)
                .push_bind(role_menu.menu_id)
                .push_bind(role_menu.menu_name.as_ref());
        });
        query_builder.build().execute(tx.as_mut()).await?;
        Ok(())
    }

    pub async fn filter_role_menu_by_role_id(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        role_id: i64,
    ) -> Result<Vec<RoleMenu>, sqlx::Error> {
        let role_menus = sqlx::query_as::<_, RoleMenu>(
            r#"
            SELECT role_id, role_name, menu_id, menu_name
            FROM role_menu
            WHERE role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(tx.as_mut())
        .await?;
        Ok(role_menus)
    }
}
