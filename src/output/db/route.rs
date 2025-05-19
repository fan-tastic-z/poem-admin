use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Transaction};

use crate::{domain::models::route::Route, errors::Error};

use super::db::Db;

impl Db {
    pub async fn filter_route_by_menu_ids(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        menu_ids: Vec<i64>,
    ) -> Result<Vec<Route>, Error> {
        let routes = sqlx::query_as::<_, Route>(
            r#"
            SELECT
                name,
                method,
                menu_id,
                menu_name
            FROM
                route
            WHERE
                menu_id = ANY($1)
        "#,
        )
        .bind(menu_ids)
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch routes".to_string()))?;
        Ok(routes)
    }
}
