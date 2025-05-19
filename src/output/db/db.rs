use std::{fmt, sync::Arc};

use error_stack::{Result, ResultExt};
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use sqlx_adapter::{
    SqlxAdapter,
    casbin::{self, CoreApi, DefaultModel, Enforcer, RbacApi},
};
use tokio::sync::RwLock;

use crate::{
    config::config::Config,
    domain::models::route::{RouteMethod, RoutePath},
    errors::Error,
};

const ACL_MODEL: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m =  r.obj == p.obj && r.act == p.act && g(r.sub, p.sub)
"#;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
    pub enforcer: EnforcerWrapper,
}

impl Db {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let opts = PgConnectOptions::new()
            .host(&config.database.host)
            .port(config.database.port)
            .username(&config.database.username)
            .password(&config.database.password)
            .database(&config.database.database_name);
        let pool = PgPoolOptions::new()
            .connect_with(opts)
            .await
            .change_context_lazy(|| Error::Message("failed to connect to database".to_string()))?;
        let model = DefaultModel::from_str(ACL_MODEL)
            .await
            .change_context_lazy(|| Error::Message("failed to load casbin model".to_string()))?;
        let adapter = SqlxAdapter::new_with_pool(pool.clone())
            .await
            .change_context_lazy(|| {
                Error::Message("failed to create casbin adapter".to_string())
            })?;
        let mut enforcer = casbin::Enforcer::new(model, adapter)
            .await
            .change_context_lazy(|| {
                Error::Message("failed to create casbin enforcer".to_string())
            })?;
        enforcer
            .load_policy()
            .await
            .change_context_lazy(|| Error::Message("failed to load casbin policy".to_string()))?;

        Ok(Self {
            pool,
            enforcer: EnforcerWrapper(Arc::new(RwLock::new(enforcer))),
        })
    }
}

const ROLE_PREFIX: &str = "role:";
const USER_PREFIX: &str = "user:";

#[derive(Clone)]
pub struct EnforcerWrapper(Arc<RwLock<Enforcer>>);

impl EnforcerWrapper {
    pub fn new(enforcer: Enforcer) -> Self {
        Self(Arc::new(RwLock::new(enforcer)))
    }

    pub async fn check_permission(
        &self,
        user_id: i64,
        path: &RoutePath,
        method: &RouteMethod,
    ) -> Result<bool, Error> {
        if user_id == 1 {
            return Ok(true);
        }
        let user = format!("{USER_PREFIX}{user_id}");
        let res = self
            .0
            .read()
            .await
            .enforce(vec![user, path.to_string(), method.to_string()])
            .change_context_lazy(|| {
                Error::Message("failed to check casbin permission".to_string())
            })?;
        Ok(res)
    }

    pub async fn add_role_for_user(&self, user_id: &str, role_id: &str) -> Result<(), Error> {
        let user = format!("{USER_PREFIX}{user_id}");
        let role = format!("{ROLE_PREFIX}{role_id}");
        self.0
            .write()
            .await
            .add_role_for_user(&user, &role, None)
            .await
            .change_context_lazy(|| Error::Message("failed to add casbin role".to_string()))?;
        Ok(())
    }

    pub async fn add_permissions_for_role(
        &self,
        role_id: &str,
        permissions: Vec<Vec<String>>,
    ) -> Result<(), Error> {
        let role = format!("{ROLE_PREFIX}{role_id}");
        self.0
            .write()
            .await
            .add_permissions_for_user(&role, permissions)
            .await
            .change_context_lazy(|| {
                Error::Message("failed to add casbin permissions".to_string())
            })?;
        Ok(())
    }
}

impl fmt::Debug for EnforcerWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnforcerWrapper")
            .field("enforcer", &"Enforcer")
            .finish()
    }
}
