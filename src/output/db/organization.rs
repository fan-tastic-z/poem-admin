use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::{
        organization::{CreateOrganizationRequest, Organization, OrganizationName},
        page_utils::PageFilter,
    },
    errors::Error,
};

use super::base::{
    Dao, DaoQueryBuilder, dao_create, dao_fetch_all, dao_fetch_by_column, dao_fetch_by_id,
};

pub struct OrganizationDao;

impl Dao for OrganizationDao {
    const TABLE: &'static str = "organization";
}

impl OrganizationDao {
    pub async fn fetch_by_id(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<Organization, Error> {
        dao_fetch_by_id::<Self, _>(tx, id).await
    }

    pub async fn all_organizations(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<Organization>, Error> {
        dao_fetch_all::<Self, _>(tx).await
    }

    pub async fn save_organization(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateOrganizationRequest,
    ) -> Result<i64, Error> {
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }

    pub async fn fetch_organization_by_name(
        tx: &mut Transaction<'_, Postgres>,
        name: &OrganizationName,
    ) -> Result<Option<Organization>, Error> {
        dao_fetch_by_column::<Self, _>(tx, "name", name.as_ref()).await
    }

    pub async fn filter_organizations(
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        page_filter: &PageFilter,
    ) -> Result<Vec<Organization>, Error> {
        let organizations = DaoQueryBuilder::<Self>::new()
            .and_where_like("name", name)
            .order_by_desc("id")
            .limit_offset(
                *page_filter.page_size().as_ref() as i64,
                (*page_filter.page_no().as_ref() as i64 - 1)
                    * *page_filter.page_size().as_ref() as i64,
            )
            .fetch_all(tx)
            .await
            .change_context_lazy(|| Error::Message("failed to filter organizations".to_string()))?;
        Ok(organizations)
    }
}
