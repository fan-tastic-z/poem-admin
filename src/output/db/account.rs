use error_stack::{Result, ResultExt};
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::{
        account::{Account, AccountName, AccountPassword, CreateAccountRequest},
        page_utils::PageFilter,
    },
    errors::Error,
    utils::password_hash::compute_password_hash,
};

use super::base::{
    Dao, DaoQueryBuilder, dao_batch_insert, dao_batch_upsert, dao_create, dao_fetch_by_column,
    dao_fetch_by_id, dao_list_by_array_column, dao_upsert,
};

pub struct AccountDao;

impl Dao for AccountDao {
    const TABLE: &'static str = "account";
}

impl AccountDao {
    pub async fn create_account(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateAccountRequest,
    ) -> Result<i64, Error> {
        let password = compute_password_hash(&req.password)?;
        let req = req.with_password(
            AccountPassword::try_from(password)
                .change_context_lazy(|| Error::Message("failed to create account".to_string()))?,
        );
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }

    pub async fn create_super_user(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateAccountRequest,
    ) -> Result<i64, Error> {
        let password = compute_password_hash(&req.password)?;
        let req = req.with_password(
            AccountPassword::try_from(password)
                .change_context_lazy(|| Error::Message("failed to create account".to_string()))?,
        );
        let id = dao_upsert::<Self, _>(tx, req, "name", &["password"]).await?;
        Ok(id)
    }

    pub async fn list_by_organization_ids(
        tx: &mut Transaction<'_, Postgres>,
        organization_ids: &[i64],
    ) -> Result<Vec<Account>, Error> {
        dao_list_by_array_column::<Self, Account>(tx, "organization_id", organization_ids).await
    }

    pub async fn filter_accounts(
        tx: &mut Transaction<'_, Postgres>,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
        page_filter: &PageFilter,
    ) -> Result<Vec<Account>, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        // 添加条件
        if let Some(name) = account_name {
            query_builder = query_builder.and_where_like("name", name.as_ref());
        } else if let Some(org_id) = organization_id {
            query_builder = query_builder.and_where_eq("organization_id", org_id);
        }

        // 组织权限限制
        if !first_level_organization_ids.is_empty() {
            query_builder =
                query_builder.and_where_in("organization_id", first_level_organization_ids);
        }

        // 分页和排序
        let page_no = *page_filter.page_no().as_ref();
        let page_size = *page_filter.page_size().as_ref();
        let offset = (page_no - 1) * page_size;

        query_builder
            .order_by_desc("id")
            .limit_offset(page_size as i64, offset as i64)
            .fetch_all(tx)
            .await
    }
    pub async fn filter_accounts_count(
        tx: &mut Transaction<'_, Postgres>,
        account_name: Option<&AccountName>,
        organization_id: Option<i64>,
        first_level_organization_ids: &[i64],
    ) -> Result<i64, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        // 添加相同的条件
        if let Some(name) = account_name {
            query_builder = query_builder.and_where_like("name", name.as_ref());
        } else if let Some(org_id) = organization_id {
            query_builder = query_builder.and_where_eq("organization_id", org_id);
        }

        if !first_level_organization_ids.is_empty() {
            query_builder =
                query_builder.and_where_in("organization_id", first_level_organization_ids);
        }

        query_builder.count(tx).await
    }

    pub async fn fetch_by_id(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<Account, Error> {
        dao_fetch_by_id::<Self, Account>(tx, id).await
    }

    // 迁移：按名称查询账户
    pub async fn fetch_by_name(
        tx: &mut Transaction<'_, Postgres>,
        name: &AccountName,
    ) -> Result<Option<Account>, Error> {
        dao_fetch_by_column::<Self, Account>(tx, "name", name.as_ref()).await
    }

    // 批量创建账户
    pub async fn batch_create_accounts(
        tx: &mut Transaction<'_, Postgres>,
        requests: Vec<CreateAccountRequest>,
    ) -> Result<Vec<i64>, Error> {
        // 对所有请求进行密码哈希处理
        let mut processed_requests = Vec::new();
        for req in requests {
            let password = compute_password_hash(&req.password)?;
            let processed_req =
                req.with_password(AccountPassword::try_from(password).change_context_lazy(
                    || Error::Message("failed to process password".to_string()),
                )?);
            processed_requests.push(processed_req);
        }

        dao_batch_insert::<Self, _>(tx, processed_requests).await
    }

    // 批量创建或更新账户
    pub async fn batch_upsert_accounts(
        tx: &mut Transaction<'_, Postgres>,
        requests: Vec<CreateAccountRequest>,
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Vec<i64>, Error> {
        // 对所有请求进行密码哈希处理
        let mut processed_requests = Vec::new();
        for req in requests {
            let password = compute_password_hash(&req.password)?;
            let processed_req =
                req.with_password(AccountPassword::try_from(password).change_context_lazy(
                    || Error::Message("failed to process password".to_string()),
                )?);
            processed_requests.push(processed_req);
        }

        dao_batch_upsert::<Self, _>(tx, processed_requests, conflict_column, update_columns).await
    }
}
