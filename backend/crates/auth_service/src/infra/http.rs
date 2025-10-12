use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use common::{
    auth::{AuthUser, JwtKeys, Role, ensure_user_role, issue_jwt},
    config::AppConfig,
    error::AppResult,
};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::repo_sqlx::SqlxAuthRepo;
use crate::{
    app::AuthService,
    domain::{
        AccessTokenResp, DoctorProfileResp, DoctorSignupReq, LoginDoctorReq, LoginPatientReq,
        MedicalRightItem, PatientProfileResp, PatientSignupReq,
    },
};

const ACCESS_TOKEN_TTL_MINUTES: i64 = 60 * 24;

#[derive(Clone)]
pub struct Ctx {
    svc: AuthService<SqlxAuthRepo>,
    jwt: JwtKeys,
    pool: PgPool,
}
impl Ctx {
    pub fn new(pool: PgPool, jwt: JwtKeys) -> Self {
        Self {
            svc: AuthService::new(SqlxAuthRepo::new(pool.clone())),
            jwt,
            pool,
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
    let token = issue_jwt(user_id, &ctx.jwt, ACCESS_TOKEN_TTL_MINUTES)?;
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
    let token = issue_jwt(user_id, &ctx.jwt, ACCESS_TOKEN_TTL_MINUTES)?;
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
    get,
    path = "/users/medical_rights",
    responses((status = 200, description = "User medical rights", body = [MedicalRightItem])),
    tag = "auth",
    security(("bearerAuth" = []))
)]
async fn get_my_medical_rights(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<MedicalRightItem>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;

    let items = ctx.svc.user_medical_rights(user_id).await?;
    Ok(Json(items))
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
    let token = issue_jwt(user_id, &ctx.jwt, ACCESS_TOKEN_TTL_MINUTES)?;
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
    let token = issue_jwt(user_id, &ctx.jwt, ACCESS_TOKEN_TTL_MINUTES)?;
    Ok(Json(AccessTokenResp {
        access_token: token,
    }))
}

#[utoipa::path(
    post,
    path = "/users/refresh",
    responses((status = 200, description = "Refreshed access token", body = AccessTokenResp)),
    tag = "auth",
    security(("bearerAuth" = []))
)]
async fn refresh_access_token(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<AccessTokenResp>> {
    let token = issue_jwt(user_id, &ctx.jwt, ACCESS_TOKEN_TTL_MINUTES)?;
    Ok(Json(AccessTokenResp {
        access_token: token,
    }))
}

#[utoipa::path(
    get,
    path = "/users/doctor/profiles",
    responses((status = 200, description = "Doctor profile", body = DoctorProfileResp)),
    tag = "auth",
    security(("bearerAuth" = []))
)]
async fn get_doctor_profile(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<DoctorProfileResp>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;

    let profile = ctx.svc.doctor_profile(user_id).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    get,
    path = "/users/patient/profiles",
    responses((status = 200, description = "Patient profile", body = PatientProfileResp)),
    tag = "auth",
    security(("bearerAuth" = []))
)]
async fn get_patient_profile(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<PatientProfileResp>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;

    let profile = ctx.svc.patient_profile(user_id).await?;
    Ok(Json(profile))
}

pub fn router(pool: PgPool) -> Router {
    let cfg = AppConfig::from_env();
    let jwt = common::auth::JwtKeys::from_secret(&cfg.jwt_secret);
    let ctx = Ctx::new(pool, jwt.clone());
    Router::new()
        .route("/users/patients", post(create_patient))
        .route("/users/login/patients", post(login_patient))
        .route("/users/patient/profiles", get(get_patient_profile))
        .route("/users/me/medical_rights", get(get_my_medical_rights))
        .route("/users/doctors", post(create_doctor))
        .route("/users/login/doctors", post(login_doctor))
        .route("/users/doctor/profiles", get(get_doctor_profile))
        .route("/users/medical_rights", post(upsert_medical_rights))
        .route("/users/refresh", post(refresh_access_token))
        .with_state(ctx)
        .layer(Extension(jwt))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(
        create_patient,
        login_patient,
        get_patient_profile,
        get_my_medical_rights,
        create_doctor,
        login_doctor,
        get_doctor_profile,
        upsert_medical_rights,
        refresh_access_token,
    ),
    components(
        schemas(
            PatientSignupReq,
            LoginPatientReq,
            MedicalRightItem,
            DoctorSignupReq,
            LoginDoctorReq,
            AccessTokenResp,
            DoctorProfileResp,
            PatientProfileResp
        )
    ),
    modifiers(&SecurityAddon),
    tags((name = "auth", description = "Auth/Users APIs"))
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
