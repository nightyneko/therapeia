use super::repo_sqlx::SqlxOrderRepo;
use crate::{
    app::OrderService,
    domain::{CreateOrderReq, CreateOrderResp, OrderDetail, OrderItemSummary, OrderStatus},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use common::{
    auth::{AuthUser, JwtKeys, Role, ensure_user_role},
    config::AppConfig,
    error::{AppError, AppResult},
};
use db::PgTx;
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(Clone)]
pub struct Ctx {
    pool: PgPool,
    svc: OrderService<SqlxOrderRepo>,
}

impl Ctx {
    pub fn new(pool: PgPool) -> Self {
        let svc = OrderService::new(SqlxOrderRepo::new(pool.clone()));
        Self { pool, svc }
    }

    async fn begin_tx(&self) -> AppResult<PgTx<'_>> {
        let tx = self.pool.begin().await?;
        Ok(tx)
    }
}

#[utoipa::path(
    get,
    path = "/orders",
    responses(
        (status = 200, description = "Orders", body = [OrderDetail])
    ),
    tag = "orders",
    security(("bearerAuth" = []))
)]
async fn list_orders(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<OrderDetail>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let orders = ctx.svc.list_orders(user_id).await?;
    Ok(Json(orders))
}

#[utoipa::path(
    post,
    path = "/order",
    request_body = CreateOrderReq,
    responses(
        (status = 201, description = "Order created", body = CreateOrderResp),
        (status = 400, description = "Invalid payload")
    ),
    tag = "orders",
    security(("bearerAuth" = []))
)]
async fn create_order(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<CreateOrderReq>,
) -> AppResult<(StatusCode, Json<CreateOrderResp>)> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    for item in &req.items {
        if item.amount <= 0 {
            return Err(AppError::BadRequest("amount must be positive".into()));
        }
    }
    let mut tx = ctx.begin_tx().await?;
    let order_id = ctx.svc.create_order(&mut tx, user_id, &req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(CreateOrderResp { order_id })))
}

#[utoipa::path(
    post,
    path = "/order/{order_id}/confirm",
    params(("order_id" = i32, Path)),
    responses(
        (status = 204, description = "Order confirmed"),
        (status = 404, description = "Order not found"),
    ),
    tag = "orders",
    security(("bearerAuth" = []))
)]
async fn confirm_order(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(order_id): Path<i32>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let mut tx = ctx.begin_tx().await?;
    ctx.svc.confirm_order(&mut tx, user_id, order_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router(pool: PgPool) -> Router {
    let ctx = Ctx::new(pool.clone());
    let cfg = AppConfig::from_env();
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);
    Router::new()
        .route("/orders", get(list_orders))
        .route("/order", post(create_order))
        .route("/order/{order_id}/confirm", post(confirm_order))
        .with_state(ctx)
        .layer(Extension(jwt_keys))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(list_orders, create_order, confirm_order),
    components(schemas(
        OrderDetail,
        OrderItemSummary,
        OrderStatus,
        CreateOrderReq,
        CreateOrderResp
    )),
    modifiers(&SecurityAddon),
    tags((name = "orders", description = "Order APIs"))
)]
pub struct ApiDoc;

pub struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::{
            Components,
            security::{Http, HttpAuthScheme, SecurityScheme},
        };
        let components = openapi.components.get_or_insert(Components::default());
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
