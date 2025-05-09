use poem::{Result, handler, http::StatusCode, web::Data};
use serde::Serialize;

use crate::{
    cli::Ctx,
    domain::{models::menu::MenuTree, ports::SysService},
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListMenuResponseData {
    pub menus: Vec<MenuTree>,
}

#[handler]
pub async fn list_menu<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
) -> Result<ApiSuccess<ListMenuResponseData>, ApiError> {
    state
        .sys_service
        .list_menu()
        .await
        .map_err(ApiError::from)
        .map(|ref menu_trees| {
            ApiSuccess::new(
                StatusCode::OK,
                ListMenuResponseData {
                    menus: menu_trees.to_vec(),
                },
            )
        })
}
