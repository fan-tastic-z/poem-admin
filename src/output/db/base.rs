use error_stack::{Result, ResultExt};
use modql::{SIden, field::HasSeaFields};
use sea_query::{
    Alias, Asterisk, Condition, Expr, Iden, IntoIden, OnConflict, PostgresQueryBuilder, Query,
    SelectStatement, TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, Postgres, Row, Transaction};

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

// dao_batch_insert common batch insert method for all dao
pub async fn dao_batch_insert<D, E>(
    tx: &mut Transaction<'_, Postgres>,
    requests: Vec<E>,
) -> Result<Vec<i64>, Error>
where
    E: HasSeaFields + Clone,
    D: Dao,
{
    if requests.is_empty() {
        return Ok(Vec::new());
    }

    // 获取第一个请求的字段结构来确定列名
    let first_fields = requests[0].clone().not_none_sea_fields();
    let (columns, _) = first_fields.for_sea_insert();

    let mut query = Query::insert();
    query.into_table(D::table_ref()).columns(columns);

    // 为每个请求添加值
    for req in requests {
        let fields = req.not_none_sea_fields();
        let (_, sea_values) = fields.for_sea_insert();
        query.values(sea_values).change_context_lazy(|| {
            Error::Message("failed to add batch insert values".to_string())
        })?;
    }

    query.returning(Query::returning().columns([CommonIden::Id]));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    log::debug!("sql: {} values: {:?}", sql, values);

    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let results = sqlx_query
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to batch insert records".to_string()))?;

    let ids = results.into_iter().map(|(id,)| id).collect();
    Ok(ids)
}

// dao_batch_upsert common batch upsert method for all dao
pub async fn dao_batch_upsert<D, E>(
    tx: &mut Transaction<'_, Postgres>,
    requests: Vec<E>,
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<Vec<i64>, Error>
where
    E: HasSeaFields + Clone,
    D: Dao,
{
    if requests.is_empty() {
        return Ok(Vec::new());
    }

    // 获取第一个请求的字段结构来确定列名
    let first_fields = requests[0].clone().not_none_sea_fields();
    let (columns, _) = first_fields.for_sea_insert();

    let mut query = Query::insert();
    query.into_table(D::table_ref()).columns(columns);

    // 为每个请求添加值
    for req in requests {
        let fields = req.not_none_sea_fields();
        let (_, sea_values) = fields.for_sea_insert();
        query.values(sea_values).change_context_lazy(|| {
            Error::Message("failed to add batch upsert values".to_string())
        })?;
    }

    // 添加 ON CONFLICT 处理
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
    let results = sqlx_query
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to batch upsert records".to_string()))?;

    let ids = results.into_iter().map(|(id,)| id).collect();
    Ok(ids)
}

// 通用的按ID查询方法
pub async fn dao_fetch_by_id<D, T>(tx: &mut Transaction<'_, Postgres>, id: i64) -> Result<T, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .and_where(Expr::col(CommonIden::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_one(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch record by id".to_string()))?;

    Ok(result)
}

// 通用的获取所有记录方法
pub async fn dao_fetch_all<D, T>(tx: &mut Transaction<'_, Postgres>) -> Result<Vec<T>, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch all records".to_string()))?;

    Ok(result)
}

// 通用的按单个条件查询方法
pub async fn dao_fetch_by_column<D, T>(
    tx: &mut Transaction<'_, Postgres>,
    column_name: &str,
    value: &str,
) -> Result<Option<T>, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .and_where(Expr::col(Alias::new(column_name)).eq(value))
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, values)
        .fetch_optional(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to fetch record by column".to_string()))?;

    Ok(result)
}

// 通用的按数组条件查询方法
pub async fn dao_list_by_array_column<D, T>(
    tx: &mut Transaction<'_, Postgres>,
    column_name: &str,
    values: &[i64],
) -> Result<Vec<T>, Error>
where
    D: Dao,
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
{
    let (sql, query_values) = Query::select()
        .from(D::table_ref())
        .column(Asterisk)
        .and_where(Expr::col(Alias::new(column_name)).is_in(values.iter().copied()))
        .build_sqlx(PostgresQueryBuilder);

    log::debug!("sql: {} values: {:?}", sql, query_values);

    let result = sqlx::query_as_with::<_, T, _>(&sql, query_values)
        .fetch_all(tx.as_mut())
        .await
        .change_context_lazy(|| {
            Error::Message("failed to list records by array column".to_string())
        })?;

    Ok(result)
}

// 通用的分页查询构建器
pub struct DaoQueryBuilder<D: Dao> {
    query: SelectStatement,
    conditions: Vec<Condition>,
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Dao> Default for DaoQueryBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: Dao> DaoQueryBuilder<D> {
    pub fn new() -> Self {
        let mut query = Query::select();
        query.from(D::table_ref()).column(Asterisk);

        Self {
            query,
            conditions: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn and_where_like(mut self, column: &str, value: &str) -> Self {
        let condition = Expr::col(Alias::new(column)).like(format!("%{}%", value));
        self.query.and_where(condition.clone());
        self.conditions.push(Condition::all().add(condition));
        self
    }

    pub fn and_where_eq(mut self, column: &str, value: i64) -> Self {
        let condition = Expr::col(Alias::new(column)).eq(value);
        self.query.and_where(condition.clone());
        self.conditions.push(Condition::all().add(condition));
        self
    }

    pub fn and_where_in(mut self, column: &str, values: &[i64]) -> Self {
        if !values.is_empty() {
            let condition = Expr::col(Alias::new(column)).is_in(values.iter().copied());
            self.query.and_where(condition.clone());
            self.conditions.push(Condition::all().add(condition));
        }
        self
    }

    pub fn order_by_desc(mut self, column: &str) -> Self {
        self.query
            .order_by(Alias::new(column), sea_query::Order::Desc);
        self
    }

    pub fn limit_offset(mut self, limit: i64, offset: i64) -> Self {
        self.query.limit(limit as u64).offset(offset as u64);
        self
    }

    pub async fn fetch_all<T>(self, tx: &mut Transaction<'_, Postgres>) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin + Send,
    {
        let (sql, values) = self.query.build_sqlx(PostgresQueryBuilder);
        log::debug!("sql: {} values: {:?}", sql, values);

        let result = sqlx::query_as_with::<_, T, _>(&sql, values)
            .fetch_all(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to fetch records".to_string()))?;

        Ok(result)
    }

    pub async fn count(self, tx: &mut Transaction<'_, Postgres>) -> Result<i64, Error> {
        let mut count_query = Query::select();
        count_query
            .from(D::table_ref())
            .expr(Expr::col(CommonIden::Id).count());

        // 应用存储的条件
        for condition in self.conditions {
            count_query.cond_where(condition);
        }

        let (sql, values) = count_query.build_sqlx(PostgresQueryBuilder);
        log::debug!("sql: {} values: {:?}", sql, values);

        let result = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to count records".to_string()))?;

        Ok(result.get::<i64, _>(0))
    }
}
