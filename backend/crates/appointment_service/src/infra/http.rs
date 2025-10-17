use super::repo_sqlx::SqlxAppointmentRepo;
use crate::{
    app::{AppointmentRepo, AppointmentService},
    domain::{
        Appointment, AppointmentOverview, AppointmentStatus, CreateAppointmentReq,
        DoctorAppointmentView, DoctorListItem, DoctorTimeslotView, NewAppointment,
        UpdateTimeslotReq,
    },
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use common::{
    auth::{AuthUser, JwtKeys, Role, ensure_user_role, user_has_role},
    config::AppConfig,
    error::{AppError, AppResult},
};
use sqlx::{PgPool, Row};
use time::macros::format_description;
use time::{Date, OffsetDateTime, Time, Weekday};
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Clone)]
pub struct Ctx {
    pool: PgPool,
    svc: AppointmentService<SqlxAppointmentRepo>,
}

impl Ctx {
    pub fn new(pool: PgPool) -> Self {
        let svc = AppointmentService::new(SqlxAppointmentRepo::new(pool.clone()));
        Self { pool, svc }
    }
}

#[utoipa::path(
    post,
    path = "/",
    request_body = CreateAppointmentReq,
    responses(
        (status = 200, description = "Book appointment successfully", body = Appointment),
        (status = 400, description = "Invalid payload"),
        (status = 404, description = "Doctor or timeslot not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn book(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Json(req): Json<CreateAppointmentReq>,
) -> AppResult<Json<Appointment>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let date = parse_date(&req.date)?;
    let start_time = parse_time(&req.start_time)?;
    let end_time = parse_time(&req.end_time)?;
    if start_time >= end_time {
        return Err(AppError::BadRequest(
            "start_time must be before end_time".into(),
        ));
    }
    let day_of_week = weekday_to_i32(date.weekday());

    let timeslot_id = sqlx::query_scalar!(
        r#"
        SELECT timeslot_id
        FROM time_slots
        WHERE doctor_id = $1
          AND day_of_weeks = $2
          AND start_time = $3
          AND end_time = $4
        "#,
        req.doctor_id,
        day_of_week,
        start_time,
        end_time
    )
    .fetch_optional(&ctx.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let mut tx = ctx.pool.begin().await?;
    let appt = ctx
        .svc
        .book(
            &mut tx,
            NewAppointment {
                patient_id: user_id,
                timeslot_id,
                date,
            },
        )
        .await?;
    tx.commit().await?;
    Ok(Json(appt))
}

#[utoipa::path(
    get,
    path = "/{appointment_id}",
    params(
        ("appointment_id" = i32, Path, description = "Appointment ID")
    ),
    responses(
        (status = 200, description = "Appointment found", body = Appointment),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn get_by_id(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(appointment_id): Path<i32>,
) -> AppResult<Json<Appointment>> {
    let Some(access) = sqlx::query(
        r#"
        SELECT a.patient_id, ts.doctor_id
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        WHERE a.appointment_id = $1
        "#,
    )
    .bind(appointment_id)
    .fetch_optional(&ctx.pool)
    .await?
    else {
        return Err(AppError::NotFound);
    };

    let patient_id: Uuid = access.try_get("patient_id")?;
    let doctor_id: Uuid = access.try_get("doctor_id")?;

    let mut allowed = patient_id == user_id;

    if !allowed && doctor_id == user_id {
        allowed = user_has_role(&ctx.pool, user_id, Role::Doctor).await?;
    }

    if !allowed {
        allowed = user_has_role(&ctx.pool, user_id, Role::Admin).await?;
    }

    if !allowed {
        return Err(AppError::Forbidden);
    }

    let Some(a) = ctx.svc.repo.by_id(appointment_id).await? else {
        return Err(AppError::NotFound);
    };
    Ok(Json(a))
}

#[utoipa::path(
    get,
    path = "/status",
    responses(
        (status = 200, description = "Upcoming appointments", body = [AppointmentOverview]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn patient_upcoming(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<AppointmentOverview>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let today = OffsetDateTime::now_utc().date();
    let rows = sqlx::query_as!(
        AppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            concat_ws(' ', u.first_name, u.last_name) AS "doctor_name!",
            dp.department AS "department?",
            ts.place_name,
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users u ON u.user_id = ts.doctor_id
        LEFT JOIN doctor_profile dp ON dp.user_id = ts.doctor_id
        WHERE a.patient_id = $1
          AND (
                (a.status = 'PENDING' AND a.date >= $2)
             OR (a.status = 'ACCEPTED' AND a.date >= $2)
          )
        ORDER BY a.date, ts.start_time
        "#,
        user_id,
        today
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(AppointmentOverview::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/status/others",
    responses(
        (status = 200, description = "Past or canceled appointments", body = [AppointmentOverview]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn patient_others(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<AppointmentOverview>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let today = OffsetDateTime::now_utc().date();
    let rows = sqlx::query_as!(
        AppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            concat_ws(' ', u.first_name, u.last_name) AS "doctor_name!",
            dp.department AS "department?",
            ts.place_name,
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users u ON u.user_id = ts.doctor_id
        LEFT JOIN doctor_profile dp ON dp.user_id = ts.doctor_id
        WHERE a.patient_id = $1
          AND (
                a.status IN ('CANCELED', 'REJECTED')
             OR (a.status = 'ACCEPTED' AND a.date < $2)
          )
        ORDER BY a.date DESC, ts.start_time
        "#,
        user_id,
        today
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(AppointmentOverview::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/by-date/{date}",
    params(("date" = String, Path, description = "Date (YYYY-MM-DD)")),
    responses(
        (status = 200, description = "Appointments on date", body = [AppointmentOverview]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn patient_by_date(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(date_str): Path<String>,
) -> AppResult<Json<Vec<AppointmentOverview>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let date = parse_date(&date_str)?;
    let rows = sqlx::query_as!(
        AppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            concat_ws(' ', u.first_name, u.last_name) AS "doctor_name!",
            dp.department AS "department?",
            ts.place_name,
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users u ON u.user_id = ts.doctor_id
        LEFT JOIN doctor_profile dp ON dp.user_id = ts.doctor_id
        WHERE a.patient_id = $1
          AND a.status = 'ACCEPTED'
          AND a.date = $2
        ORDER BY ts.start_time
        "#,
        user_id,
        date
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(AppointmentOverview::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/doctor",
    responses(
        (status = 200, description = "Doctors list", body = [DoctorListItem])
    ),
    tag = "appointments"
)]
async fn list_doctors(State(ctx): State<Ctx>) -> AppResult<Json<Vec<DoctorListItem>>> {
    let rows = sqlx::query_as!(
        DoctorRow,
        r#"
        SELECT
            u.user_id as doctor_id,
            concat_ws(' ', u.first_name, u.last_name) AS "doctor_name!",
            dp.department AS "department?"
        FROM users u
        JOIN user_roles ur ON ur.user_id = u.user_id AND ur.role = 'DOCTOR'
        LEFT JOIN doctor_profile dp ON dp.user_id = u.user_id
        ORDER BY "doctor_name!"
        "#
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| DoctorListItem {
                doctor_id: row.doctor_id,
                doctor_name: row.doctor_name,
                department: row.department,
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/doctor/{doctor_id}",
    params(("doctor_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Doctor timeslots", body = [DoctorTimeslotView]),
        (status = 404, description = "Timeslot not found"),
    ),
    tag = "appointments"
)]
async fn list_doctor_timeslots_public(
    State(ctx): State<Ctx>,
    Path(doctor_id): Path<Uuid>,
) -> AppResult<Json<Vec<DoctorTimeslotView>>> {
    let rows = sqlx::query_as!(
        DoctorTimeslotRow,
        r#"
        SELECT
            timeslot_id,
            day_of_weeks,
            place_name,
            start_time,
            end_time
        FROM time_slots
        WHERE doctor_id = $1
        ORDER BY day_of_weeks, start_time
        "#,
        doctor_id
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(DoctorTimeslotView::from).collect(),
    ))
}

#[utoipa::path(
    patch,
    path = "/{appointment_id}/canceled",
    params(("appointment_id" = i32, Path)),
    responses(
        (status = 204, description = "Appointment canceled"),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn cancel_appointment(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(appointment_id): Path<i32>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let mut tx = ctx.pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE appointments
        SET status = $3
        WHERE appointment_id = $1
          AND patient_id = $2
        "#,
    )
    .bind(appointment_id)
    .bind(user_id)
    .bind(AppointmentStatus::CANCELED)
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/{appointment_id}",
    params(("appointment_id" = i32, Path)),
    responses(
        (status = 204, description = "Appointment deleted"),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn delete_appointment(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(appointment_id): Path<i32>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Patient).await?;
    let mut tx = ctx.pool.begin().await?;
    let rows = sqlx::query!(
        r#"
        DELETE FROM appointments
        WHERE appointment_id = $1
          AND patient_id = $2
        "#,
        appointment_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/by-doctor/{date}",
    params(("date" = String, Path, description = "Date (YYYY-MM-DD)")),
    responses(
        (status = 200, description = "Doctor schedule for date", body = [DoctorAppointmentView]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn doctor_schedule_by_date(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(date_str): Path<String>,
) -> AppResult<Json<Vec<DoctorAppointmentView>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let date = parse_date(&date_str)?;
    let rows = sqlx::query_as!(
        DoctorAppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            a.patient_id,
            concat_ws(' ', up.first_name, up.last_name) AS "patient_name!",
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users up ON up.user_id = a.patient_id
        WHERE ts.doctor_id = $1
          AND a.date = $2
        ORDER BY ts.start_time
        "#,
        user_id,
        date
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(DoctorAppointmentView::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/request",
    responses(
        (status = 200, description = "Pending appointment requests", body = [DoctorAppointmentView]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn doctor_pending_requests(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<DoctorAppointmentView>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let rows = sqlx::query_as!(
        DoctorAppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            a.patient_id,
            concat_ws(' ', up.first_name, up.last_name) AS "patient_name!",
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users up ON up.user_id = a.patient_id
        WHERE ts.doctor_id = $1
          AND a.status = 'PENDING'
        ORDER BY a.date, ts.start_time
        "#,
        user_id
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(DoctorAppointmentView::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/assessed",
    responses(
        (status = 200, description = "Assessed appointment requests", body = [DoctorAppointmentView]),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn doctor_assessed_requests(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<DoctorAppointmentView>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let rows = sqlx::query_as!(
        DoctorAppointmentRow,
        r#"
        SELECT
            a.appointment_id,
            a.patient_id,
            concat_ws(' ', up.first_name, up.last_name) AS "patient_name!",
            a.date,
            ts.start_time,
            ts.end_time,
            a.status as "status: _"
        FROM appointments a
        JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
        JOIN users up ON up.user_id = a.patient_id
        WHERE ts.doctor_id = $1
          AND a.status <> 'PENDING'
        ORDER BY a.date DESC, ts.start_time
        "#,
        user_id
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(DoctorAppointmentView::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/timeslots",
    responses(
        (status = 200, description = "Doctor timeslots", body = [DoctorTimeslotView]),
        (status = 404, description = "Timeslot not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn list_my_timeslots(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
) -> AppResult<Json<Vec<DoctorTimeslotView>>> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let rows = sqlx::query_as!(
        DoctorTimeslotRow,
        r#"
        SELECT
            timeslot_id,
            day_of_weeks,
            place_name,
            start_time,
            end_time
        FROM time_slots
        WHERE doctor_id = $1
        ORDER BY day_of_weeks, start_time
        "#,
        user_id
    )
    .fetch_all(&ctx.pool)
    .await?;
    Ok(Json(
        rows.into_iter().map(DoctorTimeslotView::from).collect(),
    ))
}

#[utoipa::path(
    patch,
    path = "/timeslots/{timeslot_id}",
    params(("timeslot_id" = i32, Path)),
    request_body = UpdateTimeslotReq,
    responses(
        (status = 204, description = "Timeslot updated"),
        (status = 400, description = "Invalid payload"),
        (status = 404, description = "Timeslot not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn update_timeslot(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(timeslot_id): Path<i32>,
    Json(req): Json<UpdateTimeslotReq>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    if !(0..=6).contains(&req.day_of_weeks) {
        return Err(AppError::BadRequest(
            "day_of_weeks must be between 0 and 6".into(),
        ));
    }
    let start_time = parse_time(&req.start_time)?;
    let end_time = parse_time(&req.end_time)?;
    if start_time >= end_time {
        return Err(AppError::BadRequest(
            "start_time must be before end_time".into(),
        ));
    }
    let mut tx = ctx.pool.begin().await?;
    let rows = sqlx::query!(
        r#"
        UPDATE time_slots
        SET day_of_weeks = $3,
            place_name = $4,
            start_time = $5,
            end_time = $6
        WHERE timeslot_id = $1
          AND doctor_id = $2
        "#,
        timeslot_id,
        user_id,
        req.day_of_weeks,
        req.place_name,
        start_time,
        end_time
    )
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/timeslots/{timeslot_id}",
    params(("timeslot_id" = i32, Path)),
    responses(
        (status = 204, description = "Timeslot removed"),
        (status = 404, description = "Timeslot not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn remove_timeslot(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path(timeslot_id): Path<i32>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let mut tx = ctx.pool.begin().await?;
    let rows = sqlx::query!(
        r#"
        DELETE FROM time_slots
        WHERE timeslot_id = $1
          AND doctor_id = $2
        "#,
        timeslot_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/{appointment_id}/status/{action}",
    params(
        ("appointment_id" = i32, Path),
        ("action" = String, Path, description = "accept or reject")
    ),
    responses(
        (status = 204, description = "Appointment status updated"),
        (status = 400, description = "Invalid action"),
        (status = 404, description = "Appointment not found"),
    ),
    tag = "appointments",
    security(("bearerAuth" = []))
)]
async fn doctor_update_appointment_status(
    AuthUser { user_id, .. }: AuthUser,
    State(ctx): State<Ctx>,
    Path((appointment_id, action)): Path<(i32, String)>,
) -> AppResult<StatusCode> {
    ensure_user_role(&ctx.pool, user_id, Role::Doctor).await?;
    let status = match action.as_str() {
        "accept" | "ACCEPT" => AppointmentStatus::ACCEPTED,
        "reject" | "REJECT" => AppointmentStatus::REJECTED,
        _ => {
            return Err(AppError::BadRequest(
                "action must be accept or reject".into(),
            ));
        }
    };
    let mut tx = ctx.pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE appointments a
        SET status = $3
        FROM time_slots ts
        WHERE a.appointment_id = $1
          AND a.timeslot_id = ts.timeslot_id
          AND ts.doctor_id = $2
        "#,
    )
    .bind(appointment_id)
    .bind(user_id)
    .bind(status)
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn router(pool: PgPool) -> Router {
    let ctx = Ctx::new(pool.clone());
    let cfg = AppConfig::from_env();
    let jwt_keys = JwtKeys::from_secret(&cfg.jwt_secret);
    Router::new()
        .route("/appointments", post(book))
        .route(
            "/appointments/{appointment_id}",
            get(get_by_id).delete(delete_appointment),
        )
        .route("/appointments/status", get(patient_upcoming))
        .route("/appointments/status/others", get(patient_others))
        .route("/appointments/by-date/{date}", get(patient_by_date))
        .route("/appointments/doctor", get(list_doctors))
        .route(
            "/appointments/doctor/{doctor_id}",
            get(list_doctor_timeslots_public),
        )
        .route(
            "/appointments/{appointment_id}/canceled",
            patch(cancel_appointment),
        )
        .route(
            "/appointments/{appointment_id}/status/{action}",
            patch(doctor_update_appointment_status),
        )
        .route("/appointments/request", get(doctor_pending_requests))
        .route("/appointments/assessed", get(doctor_assessed_requests))
        .route(
            "/appointments/by-doctor/{date}",
            get(doctor_schedule_by_date),
        )
        .route("/appointments/timeslots", get(list_my_timeslots))
        .route(
            "/appointments/timeslots/{timeslot_id}",
            patch(update_timeslot).delete(remove_timeslot),
        )
        .with_state(ctx)
        .layer(Extension(jwt_keys))
}

#[derive(OpenApi, Default)]
#[openapi(
    paths(
        book,
        get_by_id,
        patient_upcoming,
        patient_others,
        patient_by_date,
        list_doctors,
        list_doctor_timeslots_public,
        doctor_schedule_by_date,
        doctor_pending_requests,
        doctor_assessed_requests,
        list_my_timeslots,
        update_timeslot,
        remove_timeslot,
        cancel_appointment,
        delete_appointment,
        doctor_update_appointment_status
    ),
    components(schemas(
        Appointment,
        AppointmentOverview,
        CreateAppointmentReq,
        DoctorListItem,
        DoctorTimeslotView,
        DoctorAppointmentView,
        UpdateTimeslotReq
    )),
    modifiers(&SecurityAddon),
    tags((name = "appointments", description = "Appointment APIs"))
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

#[derive(sqlx::FromRow)]
struct AppointmentRow {
    appointment_id: i32,
    doctor_name: String,
    department: Option<String>,
    place_name: String,
    date: Date,
    start_time: Time,
    end_time: Time,
    status: AppointmentStatus,
}

impl From<AppointmentRow> for AppointmentOverview {
    fn from(row: AppointmentRow) -> Self {
        Self {
            appointment_id: row.appointment_id,
            doctor_name: row.doctor_name,
            department: row.department,
            place_name: row.place_name,
            date: format_date(row.date),
            start_time: format_time(row.start_time),
            end_time: format_time(row.end_time),
            status: row.status,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DoctorRow {
    doctor_id: Uuid,
    doctor_name: String,
    department: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DoctorTimeslotRow {
    timeslot_id: i32,
    day_of_weeks: i32,
    place_name: String,
    start_time: Time,
    end_time: Time,
}

impl From<DoctorTimeslotRow> for DoctorTimeslotView {
    fn from(row: DoctorTimeslotRow) -> Self {
        Self {
            timeslot_id: row.timeslot_id,
            day_of_weeks: row.day_of_weeks,
            place_name: row.place_name,
            start_time: format_time(row.start_time),
            end_time: format_time(row.end_time),
        }
    }
}

#[derive(sqlx::FromRow)]
struct DoctorAppointmentRow {
    appointment_id: i32,
    patient_id: Uuid,
    patient_name: String,
    date: Date,
    start_time: Time,
    end_time: Time,
    status: AppointmentStatus,
}

impl From<DoctorAppointmentRow> for DoctorAppointmentView {
    fn from(row: DoctorAppointmentRow) -> Self {
        Self {
            appointment_id: row.appointment_id,
            patient_id: row.patient_id,
            patient_name: row.patient_name,
            date: format_date(row.date),
            start_time: format_time(row.start_time),
            end_time: format_time(row.end_time),
            status: row.status,
            status_code: status_to_code(row.status),
        }
    }
}

fn parse_date(value: &str) -> AppResult<Date> {
    let fmt = format_description!("[year]-[month]-[day]");
    Date::parse(value, &fmt)
        .map_err(|_| AppError::BadRequest("date must be in YYYY-MM-DD format".into()))
}

fn parse_time(value: &str) -> AppResult<Time> {
    let fmt_hms = format_description!("[hour]:[minute]:[second]");
    let fmt_hm = format_description!("[hour]:[minute]");
    Time::parse(value, &fmt_hms)
        .or_else(|_| Time::parse(value, &fmt_hm))
        .map_err(|_| AppError::BadRequest("time must be in HH:MM or HH:MM:SS format".into()))
}

fn format_date(date: Date) -> String {
    let fmt = format_description!("[year]-[month]-[day]");
    date.format(&fmt).expect("valid date format")
}

fn format_time(time: Time) -> String {
    let fmt = format_description!("[hour]:[minute]");
    time.format(&fmt).expect("valid time format")
}

fn weekday_to_i32(weekday: Weekday) -> i32 {
    match weekday {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6,
    }
}

fn status_to_code(status: AppointmentStatus) -> i32 {
    match status {
        AppointmentStatus::ACCEPTED => 1,
        AppointmentStatus::PENDING => 2,
        AppointmentStatus::REJECTED => 3,
        AppointmentStatus::CANCELED => 4,
    }
}
