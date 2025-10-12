use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use common::{
    auth::{JwtKeys, issue_jwt},
    config::AppConfig,
    error::AppResult,
};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::repo_sqlx::SqlxAuthRepo;
use crate::{
    app::AuthService,
    domain::{
        AccessTokenResp, DoctorSignupReq, LoginDoctorReq, LoginPatientReq, MedicalRightItem,
        PatientSignupReq,
    },
};

#[derive(Clone)]
pub struct Ctx {
    svc: AuthService<SqlxAuthRepo>,
    jwt: JwtKeys,
}
impl Ctx {
    pub fn new(pool: PgPool, jwt: JwtKeys) -> Self {
        Self {
            svc: AuthService::new(SqlxAuthRepo::new(pool)),
            jwt,
        }
    }
}

#[utoipa::path(
    post,
    path = "/users/patients",
    request_body = PatientSignupReq,
    responses((status = 201, description = "Created", body = AccessTokenResp)),
    tag = "auth"
)]
async fn create_patient(
    State(ctx): State<Ctx>,
    Json(req): Json<PatientSignupReq>,
) -> AppResult<(StatusCode, Json<AccessTokenResp>)> {
    let user_id = ctx.svc.create_patient(req.into()).await?;
    let token = issue_jwt(user_id, &ctx.jwt, 60)?;
    Ok((
        StatusCode::CREATED,
        Json(AccessTokenResp {
            access_token: token,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/users/login/patients",
    request_body = LoginPatientReq,
    responses((status = 200, description = "OK", body = AccessTokenResp)),
    tag = "auth"
)]
async fn login_patient(
    State(ctx): State<Ctx>,
    Json(req): Json<LoginPatientReq>,
) -> AppResult<Json<AccessTokenResp>> {
    let user_id = ctx.svc.login_patient(req.into()).await?;
    let token = issue_jwt(user_id, &ctx.jwt, 60)?;
    Ok(Json(AccessTokenResp {
        access_token: token,
    }))
}

#[utoipa::path(
    post,
    path = "/users/medical_rights",
    request_body = [MedicalRightItem],
    responses((status = 204, description = "Upserted")),
    tag = "auth"
)]
async fn upsert_medical_rights(
    State(ctx): State<Ctx>,
    Json(items): Json<Vec<MedicalRightItem>>,
) -> AppResult<StatusCode> {
    let upserts = items.into_iter().map(Into::into).collect();
    ctx.svc.upsert_medical_rights(upserts).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/users/doctors",
    request_body = DoctorSignupReq,
    responses((status = 201, description = "Created", body = AccessTokenResp)),
    tag = "auth"
)]
async fn create_doctor(
    State(ctx): State<Ctx>,
    Json(req): Json<DoctorSignupReq>,
) -> AppResult<(StatusCode, Json<AccessTokenResp>)> {
    let user_id = ctx.svc.create_doctor(req.into()).await?;
    let token = issue_jwt(user_id, &ctx.jwt, 60)?;
    Ok((
        StatusCode::CREATED,
        Json(AccessTokenResp {
            access_token: token,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/users/login/doctors",
    request_body = LoginDoctorReq,
    responses((status = 200, description = "OK", body = AccessTokenResp)),
    tag = "auth"
)]
async fn login_doctor(
    State(ctx): State<Ctx>,
    Json(req): Json<LoginDoctorReq>,
) -> AppResult<Json<AccessTokenResp>> {
    let user_id = ctx.svc.login_doctor(req.into()).await?;
    let token = issue_jwt(user_id, &ctx.jwt, 60)?;
    Ok(Json(AccessTokenResp {
        access_token: token,
    }))
}

pub fn router(pool: PgPool) -> Router {
    let cfg = AppConfig::from_env();
    let jwt = common::auth::JwtKeys::from_secret(&cfg.jwt_secret);
    let ctx = Ctx::new(pool, jwt);
    Router::new()
        .route("/users/patients", post(create_patient))
        .route("/users/login/patients", post(login_patient))
        .route("/users/medical_rights", post(upsert_medical_rights))
        .route("/users/doctors", post(create_doctor))
        .route("/users/login/doctors", post(login_doctor))
        .with_state(ctx)
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(create_patient, login_patient, upsert_medical_rights, create_doctor, login_doctor),
    components(schemas(PatientSignupReq, LoginPatientReq, MedicalRightItem, DoctorSignupReq, LoginDoctorReq, AccessTokenResp)),
    tags((name = "auth", description = "Auth/Users APIs"))
)]
pub struct ApiDoc;
