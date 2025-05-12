use sqlx::{Postgres, QueryBuilder, Transaction};

use crate::domain::models::role::CreateRoleMenuRequest;

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
                .push_bind(role_menu.menu_id())
                .push_bind(role_menu.menu_name().as_ref());
        });
        query_builder.build().execute(tx.as_mut()).await?;
        Ok(())
    }
}
