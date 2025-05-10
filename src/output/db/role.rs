use sqlx::{Postgres, Row, Transaction};

use crate::domain::models::role::{CreateRoleRequest, Role};

use super::db::Db;

impl Db {
    pub async fn fetch_role_by_name(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        let res = sqlx::query_as::<_, Role>(
            r#"
        SELECT
            id,
            name,
            description,
            created_by,
            created_by_name,
            is_deleteable,
            created_at,
            updated_at,
            deleted_at
        FROM
            role
        WHERE
            name = $1"#,
        )
        .bind(name)
        .fetch_optional(tx.as_mut())
        .await?;
        Ok(res)
    }

    pub async fn save_role(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateRoleRequest,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query(
            r#"
        INSERT INTO
            role
                (name, description, created_by, created_by_name, is_deleteable)
        VALUES
            ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        )
        .bind(req.name().as_ref())
        .bind(req.description().map(|d| d.as_ref()))
        .bind(req.created_by())
        .bind(req.created_by_name().as_ref())
        .bind(req.is_deleteable())
        .fetch_one(tx.as_mut())
        .await?;
        let id = res.get::<i64, _>("id");
        Ok(id)
    }
}
