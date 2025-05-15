use error_stack::{Result, ResultExt};
use sqlx::{FromRow, Postgres, QueryBuilder, Row, Transaction};

use crate::{
    domain::models::{
        page_utils::PageFilter,
        role::{CreateRoleRequest, Role, RoleName},
    },
    errors::Error,
};

use super::db::Db;

impl Db {
    pub async fn filter_role_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: Option<&RoleName>,
    ) -> Result<i64, Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT COUNT(id) FROM role
            "#,
        );

        if let Some(name) = name {
            query_builder.push(" WHERE name LIKE ");
            query_builder.push_bind(format!("%{}%", name.as_ref()));
        }

        let count = query_builder
            .build()
            .fetch_one(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to filter role count".to_string()))?;
        Ok(count.get::<i64, _>("count"))
    }

    pub async fn filter_role(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> Result<Vec<Role>, Error> {
        let page_no = page_filter.page_no().as_ref();
        let page_size = page_filter.page_size().as_ref();

        let offset = (page_no - 1) * page_size;

        let mut query_builder = QueryBuilder::new(
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
                role"#,
        );

        if let Some(name) = name {
            query_builder.push(" WHERE name LIKE ");
            query_builder.push_bind(format!("%{}%", name.as_ref()));
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let roles = query_builder
            .build()
            .try_map(|row| Role::from_row(&row))
            .fetch_all(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to filter roles".to_string()))?;

        Ok(roles)
    }

    pub async fn filter_role_by_name(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
    ) -> Result<Option<Role>, Error> {
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
        .await
        .change_context_lazy(|| Error::Message("failed to fetch role by name".to_string()))?;
        Ok(res)
    }

    pub async fn save_role(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateRoleRequest,
    ) -> Result<i64, Error> {
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
        .bind(req.name.as_ref())
        .bind(req.description.as_ref().map(|d| d.as_ref()))
        .bind(req.created_by)
        .bind(req.created_by_name.as_ref())
        .bind(req.is_deleteable)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save role".to_string()))?;
        let id = res.get::<i64, _>("id");
        Ok(id)
    }
}
