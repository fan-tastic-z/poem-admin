use error_stack::{Result, ResultExt};
use sqlx::{Postgres, QueryBuilder, Row, Transaction};

use crate::{
    domain::models::{
        account::{Account, AccountName, AccountPassword, CreateAccountRequest},
        page_utils::PageFilter,
    },
    errors::Error,
    utils::password_hash::compute_password_hash,
};

use super::database::Db;

impl Db {
    pub async fn list_account_by_organization_ids(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        organization_ids: &[i64],
    ) -> Result<Vec<Account>, Error> {
        let res = sqlx::query_as::<_, Account>(
            r#"
            SELECT
                id,
                name,
                password,
                email,
                phone,
                is_deletable,
                organization_id,
                organization_name,
                role_id,
                role_name
            FROM account
            WHERE organization_id = ANY($1)
            "#,
        )
        .bind(organization_ids)
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| {
            Error::Message("failed to list account by organization ids".to_string())
        })?;
        Ok(res)
    }
    pub async fn filter_account(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
        page_filter: &PageFilter,
    ) -> Result<Vec<Account>, Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                id,
                name,
                password,
                email,
                phone,
                is_deletable,
                organization_id,
                organization_name,
                role_id,
                role_name
            FROM account
            "#,
        );

        // Build WHERE conditions based on the requirements
        let mut has_where = false;

        if let Some(name) = account_name {
            query_builder.push(" WHERE name LIKE ");
            query_builder.push_bind(format!("%{}%", name.as_ref()));
            has_where = true;
        } else if let Some(org_id) = organization_id {
            query_builder.push(" WHERE organization_id = ");
            query_builder.push_bind(org_id);
            has_where = true;
        }

        // 只有当first_level_organization_ids不为空时才应用组织权限限制
        // 如果为空，说明是超级管理员，可以访问所有组织
        if !first_level_organization_ids.is_empty() {
            if has_where {
                query_builder.push(" AND organization_id = ANY(");
            } else {
                query_builder.push(" WHERE organization_id = ANY(");
            }
            query_builder.push_bind(first_level_organization_ids);
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY id DESC");

        let page_no = page_filter.page_no().as_ref();
        let page_size = page_filter.page_size().as_ref();

        let offset = (page_no - 1) * page_size;
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let accounts = query_builder
            .build_query_as::<Account>()
            .fetch_all(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to filter accounts".to_string()))?;

        Ok(accounts)
    }

    pub async fn filter_account_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
    ) -> Result<i64, Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT COUNT(id) FROM account
            "#,
        );

        // Build WHERE conditions based on the requirements
        let mut has_where = false;

        if let Some(name) = account_name {
            query_builder.push(" WHERE name LIKE ");
            query_builder.push_bind(format!("%{}%", name.as_ref()));
            has_where = true;
        } else if let Some(org_id) = organization_id {
            query_builder.push(" WHERE organization_id = ");
            query_builder.push_bind(org_id);
            has_where = true;
        }

        // 只有当first_level_organization_ids不为空时才应用组织权限限制
        // 如果为空，说明是超级管理员，可以访问所有组织
        if !first_level_organization_ids.is_empty() {
            if has_where {
                query_builder.push(" AND organization_id = ANY(");
            } else {
                query_builder.push(" WHERE organization_id = ANY(");
            }
            query_builder.push_bind(first_level_organization_ids);
            query_builder.push(")");
        }

        let count = query_builder
            .build()
            .fetch_one(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to filter account count".to_string()))?;

        Ok(count.get::<i64, _>("count"))
    }

    pub async fn fetch_account_by_id(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<Account, Error> {
        let res = sqlx::query_as::<_, Account>(
            r#"
        SELECT
            id,
            name,
            password,
            email,
            phone,
            is_deletable,
            organization_id,
            organization_name,
            role_id,
            role_name
        FROM account WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch account by id".to_string()))?;
        Ok(res)
    }

    pub async fn filter_account_by_name(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &AccountName,
    ) -> Result<Option<Account>, Error> {
        let res = sqlx::query_as::<_, Account>(
            r#"
        SELECT
            id,
            name,
            password,
            email,
            phone,
            is_deletable,
            organization_id,
            organization_name,
            role_id,
            role_name
        FROM account WHERE name = $1"#,
        )
        .bind(name.as_ref())
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to filter account by name".to_string()))?;
        Ok(res)
    }

    pub async fn save_super_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &AccountName,
        password: &AccountPassword,
    ) -> Result<i64, Error> {
        let organization_id = -1;
        let organization_name = "根组织".to_string();
        let role_id = 0;
        let res = sqlx::query(
            r#"
        INSERT INTO
            account
                (name, password, is_deletable,organization_id, organization_name, role_id)
        VALUES
            ($1, $2, $3, $4, $5, $6)
        ON conflict(name) DO UPDATE
        SET "password" = excluded.password
        RETURNING id
        "#,
        )
        .bind(name.as_ref())
        .bind(password.as_ref())
        .bind(false)
        .bind(organization_id)
        .bind(organization_name)
        .bind(role_id)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save super user".to_string()))?;

        let id = res.get::<i64, _>("id");
        Ok(id)
    }

    pub async fn save_account(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateAccountRequest,
    ) -> Result<i64, Error> {
        let password = compute_password_hash(&req.password)?;
        let res = sqlx::query(
            r#"
        INSERT INTO
            account
                (name, password, email, organization_id, organization_name, role_id, role_name, is_deletable)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        )
        .bind(req.name.as_ref())
        .bind(password)
        .bind(req.email.as_ref().map(|e| e.as_ref()).unwrap_or(""))
        .bind(req.organization_id)
        .bind(req.organization_name.as_ref())
        .bind(req.role_id)
        .bind(req.role_name.as_ref())
        .bind(req.is_deletable)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save account".to_string()))?;

        let id = res.get::<i64, _>("id");
        Ok(id)
    }
}
