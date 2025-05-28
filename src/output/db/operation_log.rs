use crate::{
    domain::models::{
        operation_log::{CreateOperationLogRequest, OperationLog},
        page_utils::PageFilter,
    },
    errors::Error,
};
use error_stack::{Result, ResultExt};
use sqlx::{Postgres, QueryBuilder, Row, Transaction};

use super::{
    base::{Dao, DaoQueryBuilder, dao_create},
    database::Db,
};

pub struct OperationLogDao;

impl Dao for OperationLogDao {
    const TABLE: &'static str = "operation_log";
}

impl OperationLogDao {
    pub async fn filter_operation_log(
        tx: &mut Transaction<'_, Postgres>,
        page_filter: &PageFilter,
        account_ids: &[i64],
    ) -> Result<Vec<OperationLog>, Error> {
        let operation_logs = DaoQueryBuilder::<Self>::new()
            .and_where_in("account_id", account_ids)
            .order_by_desc("id")
            .limit_offset(
                *page_filter.page_size().as_ref() as i64,
                (*page_filter.page_no().as_ref() as i64 - 1)
                    * *page_filter.page_size().as_ref() as i64,
            )
            .fetch_all(tx)
            .await
            .change_context_lazy(|| Error::Message("failed to filter operation log".to_string()))?;
        Ok(operation_logs)
    }

    pub async fn filter_operation_log_count(
        tx: &mut Transaction<'_, Postgres>,
        account_ids: &[i64],
    ) -> Result<i64, Error> {
        let count = DaoQueryBuilder::<Self>::new()
            .and_where_in("account_id", account_ids)
            .count(tx)
            .await
            .change_context_lazy(|| {
                Error::Message("failed to filter operation log count".to_string())
            })?;
        Ok(count)
    }

    pub async fn save_operation_log(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateOperationLogRequest,
    ) -> Result<i64, Error> {
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }
}

impl Db {
    pub async fn filter_operation_log(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        page_filter: &PageFilter,
        account_ids: &[i64],
    ) -> Result<Vec<OperationLog>, Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                id,
                account_id,
                account_name,
                ip_address,
                user_agent,
                operation_type,
                operation_module,
                operation_description,
                operation_result,
                created_at
            FROM operation_log
            "#,
        );

        query_builder.push(" WHERE account_id = ANY(");
        query_builder.push_bind(account_ids);
        query_builder.push(")");
        query_builder.push(" ORDER BY id DESC");

        let page_no = page_filter.page_no().as_ref();
        let page_size = page_filter.page_size().as_ref();

        let offset = (page_no - 1) * page_size;
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let operation_logs = query_builder
            .build_query_as::<OperationLog>()
            .fetch_all(tx.as_mut())
            .await
            .change_context_lazy(|| Error::Message("failed to filter operation log".to_string()))?;
        Ok(operation_logs)
    }

    pub async fn filter_operation_log_count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_ids: &[i64],
    ) -> Result<i64, Error> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                count(id)
            FROM operation_log
            "#,
        );
        query_builder.push(" WHERE account_id = ANY(");
        query_builder.push_bind(account_ids);
        query_builder.push(")");
        let count = query_builder
            .build()
            .fetch_one(tx.as_mut())
            .await
            .change_context_lazy(|| {
                Error::Message("failed to filter operation log count".to_string())
            })?;
        Ok(count.get::<i64, _>("count"))
    }

    pub async fn save_operation_log(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateOperationLogRequest,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
        INSERT INTO
            operation_log
                (account_id, account_name, ip_address, user_agent, operation_type, operation_module, operation_description, operation_result)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        )
        .bind(req.account_id)
        .bind(req.account_name.as_ref())
        .bind(req.ip_address.as_ref())
        .bind(req.user_agent.as_ref())
        .bind(req.operation_type.clone())
        .bind(req.operation_module.as_ref())
        .bind(req.operation_description.as_ref())
        .bind(req.operation_result.clone())
        .execute(tx.as_mut())
        .await
        .change_context_lazy(|| Error::Message("failed to save operation log".to_string()))?;

        Ok(())
    }
}
