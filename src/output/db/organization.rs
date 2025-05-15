use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    domain::models::organization::{CreateOrganizationRequest, Organization, OrganizationName},
    errors::Error,
};

use super::db::Db;

impl Db {
    pub async fn all_organizations(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<Organization>, Error> {
        let res = sqlx::query_as(
            r#"
        SELECT
            id,
            name,
            parent_id,
            parent_name
        FROM organization"#,
        )
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to all organizations".to_string()))?;
        Ok(res)
    }

    pub async fn save_organization(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateOrganizationRequest,
    ) -> Result<i64, Error> {
        let res = sqlx::query(
            r#"
        INSERT INTO
            organization
                (name, parent_id, parent_name)
        VALUES
            ($1, $2, $3)
        RETURNING id
        "#,
        )
        .bind(req.name.as_ref())
        .bind(req.parent_id)
        .bind(req.parent_name.as_ref().map(|n| n.as_ref()).unwrap_or(""))
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save organization".to_string()))?;

        let id = res.get::<i64, _>("id");
        Ok(id)
    }

    pub async fn fetch_organization_by_name(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        name: &OrganizationName,
    ) -> Result<Option<Organization>, Error> {
        let res = sqlx::query_as::<_, Organization>(
            r#"
        SELECT
            id,
            name,
            parent_id,
            parent_name
        FROM organization WHERE name = $1"#,
        )
        .bind(name.as_ref())
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| {
            Error::Message("failed to fetch organization by name".to_string())
        })?;
        Ok(res)
    }
}
