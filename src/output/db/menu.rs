use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{domain::models::menu::Menu, errors::Error};

use super::base::{Dao, DaoQueryBuilder};

pub struct MenuDao;

impl Dao for MenuDao {
    const TABLE: &'static str = "menu";
}

impl MenuDao {
    pub async fn list_menu(tx: &mut Transaction<'_, Postgres>) -> Result<Vec<Menu>, Error> {
        let query_builder = DaoQueryBuilder::<Self>::new();
        query_builder
            .order_by_desc("order_index")
            .fetch_all(tx)
            .await
    }
}
