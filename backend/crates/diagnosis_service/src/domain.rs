use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Diagnoses {
    pub diagnosis_id: i32,
    pub appointment_id: i32,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub symptom: String,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime, example = "2025-10-10T12:34:56Z")]
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosesResp {
    pub diagnosis_id: i32,
    pub symptom: String,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime, example = "2025-10-10T12:34:56Z")]
    pub recorded_at: OffsetDateTime,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatientInfoResp {
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub medical_conditions: Option<String>,
    pub drug_allergies: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime, example = "2025-10-10T12:34:56Z")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateDiagnosesReq {
    pub symptom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosesReq {
    pub appointment_id: i32,
    pub symptom: String,
}
