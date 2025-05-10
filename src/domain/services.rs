use crate::{domain::ports::SysRepository, errors::Error};

use super::{
    models::{menu::MenuTree, role::CreateRoleRequest},
    ports::SysService,
};
use error_stack::Result;

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: SysRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: SysRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> SysService for Service<R>
where
    R: SysRepository,
{
    async fn list_menu(&self) -> Result<Vec<MenuTree>, Error> {
        let res = self.repo.list_menu().await?;
        Ok(res)
    }

    async fn create_role(&self, req: &CreateRoleRequest) -> Result<i64, Error> {
        let res = self.repo.create_role(req).await?;
        Ok(res)
    }
}
