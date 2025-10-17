use super::repo_sqlx::SqlxShippingRepo;
use crate::{
    app::ShippingService,
    domain::{
        OrderStatus, ShippingAddressReq, ShippingAddressResp, ShippingMapPoints,
        ShippingOrderSummary, ShippingStatusTimeline,
    },
};
use anyhow::anyhow;
use axum::body::Body;
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    response::Response,
    routing::get,
};
use common::{
    auth::{AuthUser, JwtKeys, Role, ensure_user_role},
    config::AppConfig,
    error::{AppError, AppResult},
};
use reqwest::{Client, Url};
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(Clone)]
pub struct Ctx {
    pool: PgPool,
    svc: ShippingService<SqlxShippingRepo>,
    map_provider: Option<MapProvider>,
}

impl Ctx {
    fn new(pool: PgPool, map_provider: Option<MapProvider>) -> Self {
        let svc = ShippingService::new(SqlxShippingRepo::new(pool.clone()));
        Self {
            pool,
            svc,
            map_provider,
        }
    }
}

#[derive(serde::Deserialize)]
struct OrdersQuery {
    status: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/address",
    responses(
        (status = 200, description = "Shipping address", body = ShippingAddressResp),
        (status = 404, description = "Address not found"),
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn get_address(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<ShippingAddressResp>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    match ctx.svc.address(user_id).await? {
        Some(addr) => Ok(Json(addr)),
        None => Err(AppError::NotFound),
    }
}

#[utoipa::path(
    post,
    path = "/address",
    request_body = ShippingAddressReq,
    responses(
        (status = 201, description = "Address saved"),
        (status = 400, description = "Invalid payload"),
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn create_address(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<ShippingAddressReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    ctx.svc.upsert_address(user_id, &req).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    patch,
    path = "/address",
    request_body = ShippingAddressReq,
    responses(
        (status = 204, description = "Address updated"),
        (status = 404, description = "Address not found"),
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn update_address(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<ShippingAddressReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    ctx.svc.update_address(user_id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/orders",
    params(
        ("status" = i32, Query, description = "0=all, 1=pending, 2=shipping, 3=canceled, 4=success")
    ),
    responses(
        (status = 200, description = "Shipping orders", body = [ShippingOrderSummary])
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn list_orders(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Query(query): Query<OrdersQuery>,
) -> AppResult<Json<Vec<ShippingOrderSummary>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let filter = match query.status {
        None | Some(0) => None,
        Some(1) => Some(OrderStatus::PENDING),
        Some(2) => Some(OrderStatus::SHIPPING),
        Some(3) => Some(OrderStatus::CANCELED),
        Some(4) => Some(OrderStatus::SUCCESS),
        Some(_) => return Err(AppError::BadRequest("unknown status filter".into())),
    };
    let orders = ctx.svc.list_orders(user_id, filter).await?;
    Ok(Json(orders))
}

#[utoipa::path(
    get,
    path = "/orders/{order_id}/status",
    params(("order_id" = i32, Path)),
    responses(
        (status = 200, description = "Shipping status timeline", body = ShippingStatusTimeline),
        (status = 404, description = "Order not found"),
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn order_status(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(order_id): Path<i32>,
) -> AppResult<Json<ShippingStatusTimeline>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let timeline = ctx.svc.order_timeline(user_id, order_id).await?;
    Ok(Json(timeline))
}

#[utoipa::path(
    get,
    path = "/orders/{order_id}/map",
    params(("order_id" = i32, Path)),
    responses(
        (status = 200, description = "Map image", content_type = "image/png"),
        (status = 404, description = "Order not found"),
    ),
    tag = "shipping",
    security(("bearerAuth" = []))
)]
async fn order_map(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(order_id): Path<i32>,
) -> AppResult<Response> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let points = ctx.svc.map_points(user_id, order_id).await?;

    let image_bytes = if let Some(provider) = ctx.map_provider.as_ref() {
        if points.has_required_coordinates() {
            match provider.render_map(&points).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::warn!("geoapify render failed: {:?}", err);
                    PLACEHOLDER_PNG.to_vec()
                }
            }
        } else {
            tracing::warn!(
                "missing coordinates for order {}; falling back to placeholder map",
                order_id
            );
            PLACEHOLDER_PNG.to_vec()
        }
    } else {
        tracing::warn!(
            "GEOAPIFY_API_KEY not configured; falling back to placeholder map for order {}",
            order_id
        );
        PLACEHOLDER_PNG.to_vec()
    };

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("image/png"),
        )
        .body(Body::from(image_bytes))
        .map_err(|e| AppError::Other(e.into()))?;
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    Ok(response)
}

pub fn router(pool: PgPool) -> Router {
    let cfg = AppConfig::from_env();
    let map_provider = cfg.geoapify_api_key.clone().map(MapProvider::new);
    let ctx = Ctx::new(pool.clone(), map_provider);
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);
    Router::new()
        .route(
            "/shipping/address",
            get(get_address).post(create_address).patch(update_address),
        )
        .route("/shipping/orders", get(list_orders))
        .route("/shipping/orders/{order_id}/status", get(order_status))
        .route("/shipping/orders/{order_id}/map", get(order_map))
        .with_state(ctx)
        .layer(Extension(jwt_keys))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(get_address, create_address, update_address, list_orders, order_status, order_map),
    components(schemas(
        OrderStatus,
        ShippingAddressReq,
        ShippingAddressResp,
        ShippingOrderSummary,
        ShippingStatusTimeline
    )),
    modifiers(&SecurityAddon),
    tags((name = "shipping", description = "Shipping APIs"))
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

#[derive(Clone)]
struct MapProvider {
    client: Client,
    api_key: String,
}

impl MapProvider {
    fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    async fn render_map(&self, points: &ShippingMapPoints) -> AppResult<Vec<u8>> {
        let (ship_lat, ship_lon) = points
            .shipment_coordinates()
            .ok_or_else(|| AppError::BadRequest("missing shipment coordinates".into()))?;
        let (addr_lat, addr_lon) = points
            .address_coordinates()
            .ok_or_else(|| AppError::BadRequest("missing address coordinates".into()))?;

        // Compute map center and zoom explicitly to avoid Geoapify 400 errors

        // Marker Icon API v2 format; markers are provided as repeated query params
        let shipping_marker = format!(
            "lonlat:{:.6},{:.6};type:material;color:#ff4d4f;size:64",
            ship_lon, ship_lat
        );
        let address_marker = format!(
            "lonlat:{:.6},{:.6};type:material;color:#2196f3;size:64",
            addr_lon, addr_lat
        );

        // Determine center & zoom based on points distance
        let center_lat = (ship_lat + addr_lat) / 2.0;
        let center_lon = (ship_lon + addr_lon) / 2.0;
        let distance_km = haversine_distance_km(ship_lat, ship_lon, addr_lat, addr_lon);
        let zoom = zoom_for_distance(distance_km);

        let mut params = vec![
            ("style".to_string(), "osm-bright".to_string()),
            ("width".to_string(), "640".to_string()),
            ("height".to_string(), "480".to_string()),
            ("format".to_string(), "png".to_string()),
            ("apiKey".to_string(), self.api_key.clone()),
            (
                "center".to_string(),
                format!("lonlat:{:.6},{:.6}", center_lon, center_lat),
            ),
            ("zoom".to_string(), zoom.to_string()),
        ];
        params.push(("marker".to_string(), shipping_marker));
        params.push(("marker".to_string(), address_marker));

        let url = Url::parse_with_params(
            GEOAPIFY_STATIC_MAP_URL,
            params.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .map_err(|e| AppError::Other(e.into()))?;
        let redacted_url = url.as_str().replace(&self.api_key, "REDACTED");
        tracing::debug!("geoapify static map url={}", redacted_url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Other(e.into()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Other(anyhow!(
                "geoapify map request failed with status {} body {}",
                status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Other(e.into()))?;

        Ok(bytes.to_vec())
    }
}

fn zoom_for_distance(distance_km: f64) -> i32 {
    match distance_km {
        d if d <= 1.0 => 15,
        d if d <= 5.0 => 12,
        d if d <= 20.0 => 10,
        d if d <= 100.0 => 8,
        d if d <= 500.0 => 6,
        _ => 5,
    }
}

fn haversine_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_km = 6371.0;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_km * c
}

const GEOAPIFY_STATIC_MAP_URL: &str = "https://maps.geoapify.com/v1/staticmap";
const PLACEHOLDER_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 4, 0,
    0, 0, 181, 28, 12, 2, 0, 0, 0, 11, 73, 68, 65, 84, 120, 156, 99, 96, 96, 0, 0, 0, 3, 0, 1, 104,
    38, 89, 13, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];
