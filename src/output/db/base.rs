use error_stack::{Result, ResultExt};
use modql::{SIden, field::HasSeaFields};
use sea_query::{Alias, Iden, IntoIden, OnConflict, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::{Postgres, Transaction};

use crate::errors::Error;

pub trait Dao {
    const TABLE: &'static str;
    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

#[derive(Iden)]
pub enum CommonIden {
    Id,
}

// dao_create common create method for all dao
pub async fn dao_create<D, E>(tx: &mut Transaction<'_, Postgres>, req: E) -> Result<i64, Error>
where
    E: HasSeaFields,
    D: Dao,
{
    let fields = req.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();
    let mut query = Query::insert();
    query
        .into_table(D::table_ref())
        .columns(columns)
        .values(sea_values)
        .change_context_lazy(|| Error::Message("failed to create account".to_string()))?
        .returning(Query::returning().columns([CommonIden::Id]));
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);
    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let (id,) = sqlx_query
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to create account".to_string()))?;
    Ok(id)
}

// dao_upsert common upsert method for all dao
pub async fn dao_upsert<D, E>(
    tx: &mut Transaction<'_, Postgres>,
    req: E,
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<i64, Error>
where
    E: HasSeaFields,
    D: Dao,
{
    let fields = req.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    let mut query = Query::insert();
    query
        .into_table(D::table_ref())
        .columns(columns)
        .values(sea_values)
        .change_context_lazy(|| Error::Message("failed to upsert record".to_string()))?;

    let on_conflict = if update_columns.is_empty() {
        OnConflict::column(Alias::new(conflict_column))
            .do_nothing()
            .to_owned()
    } else {
        let mut on_conflict = OnConflict::column(Alias::new(conflict_column));
        for &col in update_columns {
            on_conflict = on_conflict.update_column(Alias::new(col)).to_owned();
        }
        on_conflict
    };

    query.on_conflict(on_conflict);
    query.returning(Query::returning().columns([CommonIden::Id]));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);

    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let (id,) = sqlx_query
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to upsert record".to_string()))?;

    Ok(id)
}
