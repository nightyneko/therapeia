use super::repo_sqlx::SqlxDiagnosesRepo;
use crate::{
    app::DiagnosesService,
    domain::{DiagnosesReq, DiagnosesResp, PatientInfoResp, UpdateDiagnosesReq},
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
    svc: DiagnosesService<SqlxDiagnosesRepo>,
    pool: PgPool,
}

impl Ctx {
    pub fn new(pool: PgPool) -> Self {
        let svc = DiagnosesService::new(SqlxDiagnosesRepo::new(pool.clone()));
        Self { svc, pool }
    }
}

#[utoipa::path(
    get,
    path = "/diagnoses/{patient_id}",
    params(("patient_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Diagnoses History", body = Vec<DiagnosesResp>),
        (status = 404, description = "Diagnoses not found"),
    ),
    tag = "diagnoses",
    security(("bearerAuth" = []))
)]
async fn history_by_patient(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(patient_id): Path<Uuid>,
) -> AppResult<Json<Vec<DiagnosesResp>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let rows = ctx.svc.history_by_patient(patient_id).await?;
    if rows.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/diagnoses/{patient_id}/info",
    params(("patient_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Patient information", body = Option<PatientInfoResp>),
        (status = 404, description = "not found"),
    ),
    tag = "diagnoses",
    security(("bearerAuth" = []))
)]
async fn patinet_info(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(patient_id): Path<Uuid>,
) -> AppResult<Json<Option<PatientInfoResp>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let rows = ctx.svc.patinet_info(patient_id).await?;
    if rows.is_none() {
        return Err(AppError::NotFound);
    }
    Ok(Json(rows))
}
#[utoipa::path(
    post,
    path = "/diagnoses/{patient_id}",
    params(("patient_id" = Uuid, Path)),
    request_body = DiagnosesReq,
    responses(
        (status = 201, description = "Diagnoses created"),
        (status = 404, description = "Patient not found"),
    ),
    tag = "diagnoses",
    security(("bearerAuth" = []))
)]
async fn create(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(patient_id): Path<Uuid>,
    Json(req): Json<DiagnosesReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    ctx.svc.create(req, patient_id, user_id).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    patch,
    path = "/diagnosis/{diagnosis_id}",
    params(("diagnosis_id" = i32, Path)),
    request_body = UpdateDiagnosesReq,
    responses(
        (status = 204, description = "Diagnoses updated"),
        (status = 404, description = "Diagnoses not found"),
    ),
    tag = "diagnoses",
    security(("bearerAuth" = []))
)]
async fn update(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(diagnosis_id): Path<i32>,
    Json(req): Json<UpdateDiagnosesReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    ctx.svc.update(req, diagnosis_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn router(pool: PgPool) -> Router {
    let ctx = Ctx::new(pool);
    let cfg = AppConfig::from_env();
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);
    Router::new()
        .route(
            "/diagnoses/{patient_id}",
            get(history_by_patient).post(create),
        )
        .route("/diagnosis/{diagnosis_id}", patch(update))
        .route("/diagnoses/{patient_id}/info", get(patinet_info))
        .with_state(ctx)
        .layer(Extension(jwt_keys))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(history_by_patient, patinet_info, create, update),
    components(schemas(DiagnosesResp, PatientInfoResp, DiagnosesReq, UpdateDiagnosesReq)),
    modifiers(&SecurityAddon),
    tags((name = "diagnoses", description = "Diagnoses APIs"))
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
