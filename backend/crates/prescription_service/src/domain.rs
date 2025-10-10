use serde::{Deserialize, Serialize};

use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Prescription {
    pub prescription_id: i32,
    pub patient_id: Uuid,
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
    pub medicines_id: i32,
    pub patient_id: Uuid,
    pub doctor_comment: Option<String>,
    pub dosage: String,
    pub amount: i32,
    pub on_going: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrescriptionReq {
    pub medicines_id: i32,
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
