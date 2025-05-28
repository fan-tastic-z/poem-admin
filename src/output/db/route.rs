use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{domain::models::route::Route, errors::Error};

use super::base::{Dao, DaoQueryBuilder};

pub struct RouteDao;

impl Dao for RouteDao {
    const TABLE: &'static str = "route";
}

impl RouteDao {
    pub async fn filter_by_menu_ids(
        tx: &mut Transaction<'_, Postgres>,
        menu_ids: &[i64],
    ) -> Result<Vec<Route>, Error> {
        DaoQueryBuilder::<Self>::new()
            .and_where_in("menu_id", menu_ids)
            .fetch_all(tx)
            .await
    }
}
