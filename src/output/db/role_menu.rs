use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::role_menu::{RoleMenu, SaveRoleMenuRequest},
    errors::Error,
};
use error_stack::{Result, ResultExt};

use super::base::{Dao, DaoQueryBuilder, dao_batch_insert};

pub struct RoleMenuDao;

impl Dao for RoleMenuDao {
    const TABLE: &'static str = "role_menu";
}

impl RoleMenuDao {
    pub async fn save_role_menus(
        tx: &mut Transaction<'_, Postgres>,
        req: &[SaveRoleMenuRequest],
    ) -> Result<Vec<i64>, Error> {
        dao_batch_insert::<Self, _>(tx, req.to_vec()).await
    }

    pub async fn filter_role_menu_by_role_id(
        tx: &mut Transaction<'_, Postgres>,
        role_id: i64,
    ) -> Result<Vec<RoleMenu>, Error> {
        let role_menus = DaoQueryBuilder::<Self>::new()
            .and_where_eq("role_id", role_id)
            .fetch_all(tx)
            .await
            .change_context_lazy(|| {
                Error::Message("failed to filter role menu by role id".to_string())
            })?;
        Ok(role_menus)
    }
}
