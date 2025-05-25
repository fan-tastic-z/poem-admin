use crate::{domain::models::operation_log::CreateOperationLogRequest, errors::Error};
use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Transaction};

use super::database::Db;

impl Db {
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
