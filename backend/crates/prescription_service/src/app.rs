use crate::domain::*;
use common::error::AppResult;
use db::PgTx;
use uuid::Uuid;
pub trait PrescriptionRepo: Send + Sync {
    #[expect(async_fn_in_trait)]
    async fn by_id(&self, id: Uuid) -> AppResult<Option<Prescription>>;
    #[expect(async_fn_in_trait)]
    async fn search_medicines_by_name(&self, input: &str) -> AppResult<Vec<(i32, String)>>;
    #[expect(async_fn_in_trait)]
    async fn get_medicine_info(&self, medicine_id: i32)
    -> AppResult<Option<(i32, String, String)>>;
    #[expect(async_fn_in_trait)]
    async fn create_prescription(
        &self,
        patient_id: Uuid,
        medicine_id: i32,
        dosage: String,
        amount: i32,
        on_going: bool,
        doctor_comment: Option<String>,
    ) -> AppResult<i32>;
    #[expect(async_fn_in_trait)]
    async fn update_prescription(
        &self,
        prescription_id: i32,
        medicine_id: i32,
        patient_id: Uuid,
        dosage: String,
        amount: i32,
        on_going: bool,
        doctor_comment: Option<String>,
    ) -> AppResult<()>;
    #[expect(async_fn_in_trait)]
    async fn delete_prescription(&self, prescription_id: i32) -> AppResult<()>;
}

#[derive(Clone)]
pub struct PrescriptionService<R: PrescriptionRepo> {
    pub repo: R,
}

impl<R: PrescriptionRepo> PrescriptionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}
