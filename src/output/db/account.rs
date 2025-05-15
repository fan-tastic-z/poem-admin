use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    domain::models::account::{Account, AccountName, AccountPassword, CreateAccountRequest},
    errors::Error,
    utils::password_hash::compute_password_hash,
};

use super::db::Db;

impl Db {
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
