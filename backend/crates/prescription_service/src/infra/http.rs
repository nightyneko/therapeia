use super::repo_sqlx::SqlxPrescriptionRepo;
use crate::{
    app::{PrescriptionRepo, PrescriptionService},
    domain::{
        CreatePrescriptionReq, MedicineInfo, MedicineSearchItem, Prescription, PrescriptionIdResp,
        UpdatePrescriptionReq,
    },
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    routing::get,
};
use common::error::{AppError, AppResult};
use common::{
    auth::{AuthUser, JwtKeys},
    config::AppConfig,
};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;


#[derive(Clone)]
pub struct Ctx {
    pool: PgPool,
    svc: PrescriptionService<SqlxPrescriptionRepo>,
}

impl Ctx {
    pub fn new(pool: PgPool) -> Self {
        let svc = PrescriptionService::new(SqlxPrescriptionRepo::new(pool.clone()));
        Self { pool, svc }
    }
}

#[utoipa::path(
    get,
    path = "/prescription/{id}",
    responses(
        (status = 200, description = "Prescription found", body = Prescription),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "prescriptions"
)]
async fn get_by_user_id(
    State(ctx): State<Ctx>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Prescription>> {
    let Some(a) = ctx.svc.repo.by_id(id).await? else {
        return Err(AppError::NotFound);
    };
    Ok(Json(a))
}

#[utoipa::path(
    get,
    path = "/prescriptions",
    responses(
        (status = 200, description = "Prescription found", body = Prescription),
        (status = 404, description = "Prescription not found"),
    ),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn get_by_user(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Prescription>> {
    println!("{}", user_id.to_string());
    let Some(a) = ctx.svc.repo.by_id(user_id).await? else {
        return Err(AppError::NotFound);
    };
    Ok(Json(a))
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
    AuthUser { .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(medicine_id): Path<i32>,
) -> AppResult<Json<MedicineInfo>> {
    let Some((medicine_id, medicine_name, img_link)) =
        ctx.svc.repo.get_medicine_info(medicine_id).await?
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
    AuthUser { .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<CreatePrescriptionReq>,
) -> AppResult<Json<PrescriptionIdResp>> {
    let id = ctx
        .svc
        .repo
        .create_prescription(
            req.patient_id,
            req.medicines_id,
            req.dosage,
            req.amount,
            req.on_going,
            req.doctor_comment,
        )
        .await?;
    Ok(Json(PrescriptionIdResp {
        prescription_id: id,
    }))
}



#[utoipa::path(
    patch,
    path = "/prescriptions/{prescriptions_id}",
    request_body = UpdatePrescriptionReq,
    params(("prescriptions_id" = i32, Path)),
    responses((status = 204, description = "Updated")),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn update_prescription(
    AuthUser { .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(prescriptions_id): Path<i32>,
    Json(req): Json<UpdatePrescriptionReq>,
) -> AppResult<()> {
    ctx.svc
        .repo
        .update_prescription(
            prescriptions_id,
            req.medicines_id,
            req.patient_id,
            req.dosage,
            req.amount,
            req.on_going,
            req.doctor_comment,
        )
        .await?;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "/prescriptions/{prescriptions_id}",
    params(("prescriptions_id" = i32, Path)),
    responses((status = 204, description = "Deleted")),
    tag = "prescriptions",
    security(("bearerAuth" = []))
)]
async fn delete_prescription(
    AuthUser { .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(prescriptions_id): Path<i32>,
) -> AppResult<()> {
    ctx.svc.repo.delete_prescription(prescriptions_id).await?;
    Ok(())
}

pub fn router(pool: PgPool) -> Router {
    let ctx = Ctx::new(pool);
    let cfg = AppConfig::from_env();
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);

    Router::new()
        // Collection: list & create
        .route("/prescriptions", get(get_by_user).post(create_prescription))
        .route(
            "/prescriptions/{id}",
            get(get_by_user_id)
                .patch(update_prescription)
                .delete(delete_prescription),
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
    AuthUser { .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(input): Path<String>,
) -> AppResult<Json<Vec<MedicineSearchItem>>> {
    let rows = ctx.svc.repo.search_medicines_by_name(&input).await?;
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
    components(schemas(MedicineSearchItem, MedicineInfo, CreatePrescriptionReq, UpdatePrescriptionReq, PrescriptionIdResp)),
    modifiers(&SecurityAddon),
    tags((name = "prescription", description = "Prescription APIs"))
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
