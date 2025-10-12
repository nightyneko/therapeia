use crate::domain::*;
use common::error::AppResult;
use uuid::Uuid;
pub trait DiagnosesRepo: Send + Sync {
    #[expect(async_fn_in_trait)]
    async fn list_by_patient(&self, id: Uuid) -> AppResult<Vec<DiagnosesResp>>;
    #[expect(async_fn_in_trait)]
    async fn info_by_patient(&self, id: Uuid) -> AppResult<Option<PatientInfoResp>>;
    #[expect(async_fn_in_trait)]
    async fn update_by_patient(
        &self,
        rec: UpdateDiagnosesReq,
        diagnosis_id: i32,
    ) -> AppResult<()>;
    #[expect(async_fn_in_trait)]
    async fn create_by_patient(
        &self,
        rec: DiagnosesReq,
        patient_id: Uuid,
        doctor_id: Uuid,
    ) -> AppResult<()>;
}

#[derive(Clone)]
pub struct DiagnosesService<R: DiagnosesRepo> {
    pub repo: R,
}

impl<R: DiagnosesRepo> DiagnosesService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn history_by_patient(&self, id: Uuid) -> AppResult<Vec<DiagnosesResp>> {
        self.repo.list_by_patient(id).await
    }
    pub async fn patinet_info(&self, id: Uuid) -> AppResult<Option<PatientInfoResp>> {
        self.repo.info_by_patient(id).await
    }
    pub async fn update(
        &self,
        rec: UpdateDiagnosesReq,
        diagnosis_id: i32,
       
    ) -> AppResult<()> {
        self.repo
            .update_by_patient(rec, diagnosis_id)
            .await
    }
    pub async fn create(
        &self,
        rec: DiagnosesReq,
        patient_id: Uuid,
        doctor_id: Uuid,
    ) -> AppResult<()> {
        self.repo
            .create_by_patient(rec, patient_id, doctor_id)
            .await
    }
}
