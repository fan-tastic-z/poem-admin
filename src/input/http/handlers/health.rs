use poem::handler;

#[handler]
pub async fn health() -> &'static str {
    "OK"
}
