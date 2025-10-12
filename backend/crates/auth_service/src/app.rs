use crate::domain::{
    DoctorLoginInput, DoctorSignupInput, MedicalRightUpsert, PatientLoginInput, PatientSignupInput,
};
use common::error::AppResult;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService<R: AuthRepo> {
    repo: R,
}

impl<R: AuthRepo> AuthService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_patient(&self, input: PatientSignupInput) -> AppResult<Uuid> {
        self.repo.create_patient(input).await
    }

    pub async fn login_patient(&self, input: PatientLoginInput) -> AppResult<Uuid> {
        self.repo.login_patient(input).await
    }

    pub async fn upsert_medical_rights(&self, items: Vec<MedicalRightUpsert>) -> AppResult<()> {
        self.repo.upsert_medical_rights(items).await
    }

    pub async fn create_doctor(&self, input: DoctorSignupInput) -> AppResult<Uuid> {
        self.repo.create_doctor(input).await
    }

    pub async fn login_doctor(&self, input: DoctorLoginInput) -> AppResult<Uuid> {
        self.repo.login_doctor(input).await
    }
}

pub trait AuthRepo: Send + Sync {
    #[expect(async_fn_in_trait)]
    async fn create_patient(&self, input: PatientSignupInput) -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn login_patient(&self, input: PatientLoginInput) -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn upsert_medical_rights(&self, items: Vec<MedicalRightUpsert>) -> AppResult<()>;
    #[expect(async_fn_in_trait)]
    async fn create_doctor(&self, input: DoctorSignupInput) -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn login_doctor(&self, input: DoctorLoginInput) -> AppResult<Uuid>;
}
