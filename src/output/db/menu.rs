use error_stack::Result;
use sqlx::{Postgres, Transaction, postgres::PgRow};

use super::db::Db;

impl Db {
    pub async fn list_menu(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<PgRow>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
        SELECT
            id,
            name,
            parent_id,
            parent_name
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
