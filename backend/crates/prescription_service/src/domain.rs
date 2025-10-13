use serde::{Deserialize, Serialize};

use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Prescription {
    pub prescription_id: i32,
    pub patient_id: Uuid,
    pub medicine_name: String,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
    pub doctor_comment: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Medicines {
    pub medicine_id: i32,
    pub medicine_name: String,
    pub details: String,
    pub unit_price: f32,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MedicineSearchItem {
    pub medicine_id: i32,
    pub medicine_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MedicineInfo {
    pub medicine_id: i32,
    pub medicine_name: String,
    pub img_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePrescriptionReq {
    pub medicine_id: i32,
    pub patient_id: Uuid,
    pub doctor_comment: Option<String>,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
}

#[derive(Debug, Clone)]
pub struct CreatePrescriptionInput {
    pub medicine_id: i32,
    pub patient_id: Uuid,
    pub doctor_comment: Option<String>,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrescriptionReq {
    pub medicine_id: i32,
    pub patient_id: Uuid,
    pub doctor_comment: Option<String>,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
}

#[derive(Debug, Clone)]
pub struct UpdatePrescriptionInput {
    pub prescription_id: i32,
    pub medicine_id: i32,
    pub patient_id: Uuid,
    pub doctor_comment: Option<String>,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PrescriptionIdResp {
    pub prescription_id: i32,
}

impl From<CreatePrescriptionReq> for CreatePrescriptionInput {
    fn from(value: CreatePrescriptionReq) -> Self {
        Self {
            medicine_id: value.medicine_id,
            patient_id: value.patient_id,
            doctor_comment: value.doctor_comment,
            dosage: value.dosage,
            amount: value.amount,
            on_going: value.on_going,
        }
    }
}

impl UpdatePrescriptionInput {
    pub fn from_request(prescription_id: i32, payload: UpdatePrescriptionReq) -> Self {
        Self {
            prescription_id,
            medicine_id: payload.medicine_id,
            patient_id: payload.patient_id,
            doctor_comment: payload.doctor_comment,
            dosage: payload.dosage,
            amount: payload.amount,
            on_going: payload.on_going,
        }
    }
}
