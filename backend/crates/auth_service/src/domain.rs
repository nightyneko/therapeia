use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatientSignupReq {
    pub hn: i32,
    pub citizen_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginPatientReq {
    pub hn: i32,
    pub citizen_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MedicalRightItem {
    pub mr_id: i32,
    pub name: String,
    pub details: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DoctorSignupReq {
    pub mln: String,
    pub citizen_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginDoctorReq {
    pub mln: String,
    pub citizen_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessTokenResp {
    pub access_token: String,
}
