use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::{
        page_utils::PageFilter,
        role::{Role, RoleName, SaveRoleRequest},
    },
    errors::Error,
    output::db::base::dao_create,
};

use super::base::{Dao, DaoQueryBuilder, dao_fetch_by_column, dao_fetch_by_id};

pub struct RoleDao;

impl Dao for RoleDao {
    const TABLE: &'static str = "role";
}

impl RoleDao {
    pub async fn fetch_by_id(tx: &mut Transaction<'_, Postgres>, id: i64) -> Result<Role, Error> {
        dao_fetch_by_id::<Self, Role>(tx, id).await
    }

    pub async fn fetch_by_name(
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
    ) -> Result<Option<Role>, Error> {
        dao_fetch_by_column::<Self, Role>(tx, "name", name).await
    }

    pub async fn filter_roles_count(
        tx: &mut Transaction<'_, Postgres>,
        name: Option<&RoleName>,
    ) -> Result<i64, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        if let Some(name) = name {
            query_builder = query_builder.and_where_like("name", name.as_ref());
        }

        query_builder.count(tx).await
    }

    pub async fn filter_roles(
        tx: &mut Transaction<'_, Postgres>,
        name: Option<&RoleName>,
        page_filter: &PageFilter,
    ) -> Result<Vec<Role>, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        if let Some(name) = name {
            query_builder = query_builder.and_where_like("name", name.as_ref());
        }

        let page_no = *page_filter.page_no().as_ref();
        let page_size = *page_filter.page_size().as_ref();
        let offset = (page_no - 1) * page_size;

        query_builder
            .order_by_desc("id")
            .limit_offset(page_size as i64, offset as i64)
            .fetch_all(tx)
            .await
    }

    pub async fn save_role(
        tx: &mut Transaction<'_, Postgres>,
        req: SaveRoleRequest,
    ) -> Result<i64, Error> {
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }
}
