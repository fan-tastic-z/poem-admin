use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::domain::models::menu::Menu;

use super::database::Db;

impl Db {
    pub async fn list_menu(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<Menu>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Menu>(
            r#"
        SELECT
            id,
            name,
            parent_id,
            parent_name,
            order_index,
            created_at,
            updated_at,
            deleted_at
        FROM
            menu
        ORDER BY
            order_index
        "#,
        )
        .fetch_all(tx.as_mut())
        .await?;
        Ok(rows)
    }
}
