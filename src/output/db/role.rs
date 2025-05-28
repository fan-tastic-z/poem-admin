use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    domain::models::{
        page_utils::PageFilter,
        role::{CreateRoleRequest, Role, RoleName},
    },
    errors::Error,
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
        req: &CreateRoleRequest,
        current_user_id: i64,
        current_user_name: &str,
    ) -> Result<i64, Error> {
        let res = sqlx::query(
            r#"
        INSERT INTO
            role
                (name, description, created_by, created_by_name, is_deletable)
        VALUES
            ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        )
        .bind(req.name.as_ref())
        .bind(req.description.as_ref().map(|d| d.as_ref()))
        .bind(current_user_id)
        .bind(current_user_name)
        .bind(req.is_deletable)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save role".to_string()))?;
        let id = res.get::<i64, _>("id");
        Ok(id)
    }
}
