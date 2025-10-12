use crate::domain::*;
use common::error::AppResult;
use uuid::Uuid;
pub trait PrescriptionRepo: Send + Sync {
    #[expect(async_fn_in_trait)]
    async fn by_id(&self, id: Uuid) -> AppResult<Vec<Prescription>>;
    #[expect(async_fn_in_trait)]
    async fn search_medicines_by_name(&self, input: &str) -> AppResult<Vec<(i32, String)>>;
    #[expect(async_fn_in_trait)]
    async fn get_medicine_info(&self, medicine_id: i32)
    -> AppResult<Option<(i32, String, String)>>;
    #[expect(async_fn_in_trait)]
    async fn create_prescription(&self, input: CreatePrescriptionInput) -> AppResult<i32>;
    #[expect(async_fn_in_trait)]
    async fn update_prescription(&self, input: UpdatePrescriptionInput) -> AppResult<()>;
    #[expect(async_fn_in_trait)]
    async fn delete_prescription(&self, prescription_id: i32) -> AppResult<()>;
}

#[derive(Clone)]
pub struct PrescriptionService<R: PrescriptionRepo> {
    repo: R,
}

impl<R: PrescriptionRepo> PrescriptionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn prescription_by_patient(&self, id: Uuid) -> AppResult<Vec<Prescription>> {
        self.repo.by_id(id).await
    }

    pub async fn search_medicines(&self, input: &str) -> AppResult<Vec<(i32, String)>> {
        self.repo.search_medicines_by_name(input).await
    }

    pub async fn medicine_info(
        &self,
        medicine_id: i32,
    ) -> AppResult<Option<(i32, String, String)>> {
        self.repo.get_medicine_info(medicine_id).await
    }

    pub async fn create_prescription(&self, input: CreatePrescriptionInput) -> AppResult<i32> {
        self.repo.create_prescription(input).await
    }

    pub async fn update_prescription(&self, input: UpdatePrescriptionInput) -> AppResult<()> {
        self.repo.update_prescription(input).await
    }

    pub async fn delete_prescription(&self, prescription_id: i32) -> AppResult<()> {
        self.repo.delete_prescription(prescription_id).await
    }
}
