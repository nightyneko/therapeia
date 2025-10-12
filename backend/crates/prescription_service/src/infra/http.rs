use super::repo_sqlx::SqlxPrescriptionRepo;
use crate::{
    app::PrescriptionService,
    domain::{
        CreatePrescriptionReq, MedicineInfo, MedicineSearchItem, Prescription, PrescriptionIdResp,
        UpdatePrescriptionInput, UpdatePrescriptionReq,
    },
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
};
use common::{
    auth::{AuthUser, JwtKeys, Role, ensure_user_role},
    config::AppConfig,
    error::{AppError, AppResult},
};
use sqlx::PgPool;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Clone)]
pub struct Ctx {
    svc: PrescriptionService<SqlxPrescriptionRepo>,
    pool: PgPool,
}

impl Ctx {
    pub fn new(pool: PgPool) -> Self {
        let svc = PrescriptionService::new(SqlxPrescriptionRepo::new(pool.clone()));
        Self { svc, pool }
    }
}

#[utoipa::path(
    get,
    path = "/prescriptions/patient/{patient_id}",
    params(("patient_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Prescriptions found", body = [Prescription]),
        (status = 404, description = "Prescription not found"),
    ),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn get_by_user_id(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(patient_id): Path<Uuid>,
) -> AppResult<Json<Vec<Prescription>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let rows = ctx.svc.prescription_by_patient(patient_id).await?;
    if rows.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/prescriptions",
    responses(
        (status = 200, description = "Prescriptions found", body = [Prescription]),
        (status = 404, description = "Prescription not found"),
    ),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn get_by_user(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<Prescription>>> {
    let rows = ctx.svc.prescription_by_patient(user_id).await?;
    if rows.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/prescriptions/medicines/{medicine_id}",
    params(("medicine_id" = i32, Path)),
    responses((status = 200, description = "Medicine info", body = MedicineInfo)),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn get_medicine_info(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(medicine_id): Path<i32>,
) -> AppResult<Json<MedicineInfo>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let Some((medicine_id, medicine_name, img_link)) = ctx.svc.medicine_info(medicine_id).await?
    else {
        return Err(AppError::NotFound);
    };
    Ok(Json(MedicineInfo {
        medicine_id,
        medicine_name,
        img_link,
    }))
}

#[utoipa::path(
    post,
    path = "/prescriptions",
    request_body = CreatePrescriptionReq,
    responses((status = 201, description = "Created", body = PrescriptionIdResp)),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn create_prescription(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<CreatePrescriptionReq>,
) -> AppResult<(StatusCode, Json<PrescriptionIdResp>)> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let id = ctx.svc.create_prescription(req.into()).await?;
    Ok((
        StatusCode::CREATED,
        Json(PrescriptionIdResp {
            prescription_id: id,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/prescriptions/{prescription_id}",
    request_body = UpdatePrescriptionReq,
    params(("prescription_id" = i32, Path)),
    responses((status = 204, description = "Updated")),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn update_prescription(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(prescription_id): Path<i32>,
    Json(req): Json<UpdatePrescriptionReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let input = UpdatePrescriptionInput::from_request(prescription_id, req);
    ctx.svc.update_prescription(input).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/prescriptions/{prescription_id}",
    params(("prescription_id" = i32, Path)),
    responses((status = 204, description = "Deleted")),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn delete_prescription(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(prescription_id): Path<i32>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    ctx.svc.delete_prescription(prescription_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/prescriptions/search/{input}",
    params(("input" = String, Path, description = "Search term")),
    responses(
        (status = 200, description = "Search results", body = [MedicineSearchItem])
    ),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn search_medicines(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(input): Path<String>,
) -> AppResult<Json<Vec<MedicineSearchItem>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let rows = ctx.svc.search_medicines(&input).await?;
    Ok(Json(
        rows.into_iter()
            .map(|(medicine_id, medicine_name)| MedicineSearchItem {
                medicine_id,
                medicine_name,
            })
            .collect(),
    ))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(get_by_user_id, get_by_user, search_medicines, get_medicine_info, create_prescription, update_prescription, delete_prescription),
    components(schemas(Prescription, MedicineSearchItem, MedicineInfo, CreatePrescriptionReq, UpdatePrescriptionReq, PrescriptionIdResp)),
    modifiers(&SecurityAddon),
    tags((name = "prescriptions", description = "Prescription APIs"))
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

pub fn router(pool: PgPool) -> Router {
    let ctx = Ctx::new(pool);
    let cfg = AppConfig::from_env();
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);

    Router::new()
        // Collection: list & create
        .route("/prescriptions", get(get_by_user).post(create_prescription))
        .route("/prescriptions/patient/{patient_id}", get(get_by_user_id))
        .route(
            "/prescriptions/{prescription_id}",
            patch(update_prescription).delete(delete_prescription),
        )
        // Sub-resources / utilities
        .route(
            "/prescriptions/medicines/{medicine_id}",
            get(get_medicine_info),
        )
        .route("/prescriptions/search/{input}", get(search_medicines))
        .with_state(ctx)
        .layer(Extension(jwt_keys))
}
