use serde::{Deserialize, Serialize};
use time::{Date, Time};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "appointment_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppointmentStatus {
    PENDING,
    ACCEPTED,
    REJECTED,
    CANCELED,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Appointment {
    pub appointment_id: i32,
    pub patient_id: Uuid,
    pub timeslot_id: i32,
    pub date: Date,
    pub status: AppointmentStatus,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime, example = "2025-10-10T12:34:56Z")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TimeSlot {
    pub timeslot_id: i32,
    pub doctor_id: Uuid,
    pub day_of_weeks: i32,
    #[schema(value_type = String, format = "time", example = "09:30:00")]
    pub start_time: Time,
    #[schema(value_type = String, format = "time", example = "09:30:00")]
    pub end_time: Time,
    pub place_name: String,
}

pub struct NewAppointment {
    pub patient_id: Uuid,
    pub timeslot_id: i32,
    pub date: Date,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppointmentOverview {
    pub appointment_id: i32,
    pub doctor_name: String,
    #[schema(nullable = true)]
    pub department: Option<String>,
    pub place_name: String,
    #[schema(example = "2025-09-23")]
    pub date: String,
    #[schema(example = "09:00")]
    pub start_time: String,
    #[schema(example = "12:00")]
    pub end_time: String,
    pub status: AppointmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DoctorListItem {
    pub doctor_id: Uuid,
    pub doctor_name: String,
    #[schema(nullable = true)]
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DoctorTimeslotView {
    pub timeslot_id: i32,
    pub day_of_weeks: i32,
    pub place_name: String,
    #[schema(example = "09:00")]
    pub start_time: String,
    #[schema(example = "12:00")]
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DoctorAppointmentView {
    pub appointment_id: i32,
    pub patient_id: Uuid,
    pub patient_name: String,
    #[schema(example = "2025-09-23")]
    pub date: String,
    #[schema(example = "09:00")]
    pub start_time: String,
    #[schema(example = "12:00")]
    pub end_time: String,
    pub status: AppointmentStatus,
    pub status_code: i32,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateAppointmentReq {
    pub doctor_id: Uuid,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateTimeslotReq {
    pub day_of_weeks: i32,
    pub place_name: String,
    pub start_time: String,
    pub end_time: String,
}
