use super::super::app::DiagnosesRepo;
use super::super::domain::*;
use common::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxDiagnosesRepo {
    pool: PgPool,
}
impl SqlxDiagnosesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DiagnosesRepo for SqlxDiagnosesRepo {
    async fn list_by_patient(&self, id: Uuid) -> AppResult<Vec<DiagnosesResp>> {
        let rows = sqlx::query_as!(
            DiagnosesResp,
            r#"SELECT diagnosis_id, symptom, recorded_at FROM diagnoses WHERE patient_id = $1 ORDER BY recorded_at "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
    async fn info_by_patient(&self, id: Uuid) -> AppResult<Option<PatientInfoResp>> {
        let rows = sqlx::query_as!(
            PatientInfoResp,
            r#"SELECT age as "age?",
                      gender as "gender?",
                      height_cm::float8 as "height_cm?",
                      weight_kg::float8 as "weight_kg?",
                      medical_conditions as "medical_conditions?",
                      drug_allergies as "drug_allergies?",
                      updated_at
               FROM patient_health_info
               WHERE patient_id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rows)
    }
    async fn update_by_patient(&self, rec: UpdateDiagnosesReq, diagnosis_id: i32) -> AppResult<()> {
        let rows = sqlx::query!(
            r#"UPDATE diagnoses SET symptom = $1 WHERE diagnosis_id = $2"#,
            rec.symptom,
            diagnosis_id
        )
        .execute(&self.pool)
        .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
    async fn create_by_patient(
        &self,
        rec: DiagnosesReq,
        patient_id: Uuid,
        doctor_id: Uuid,
    ) -> AppResult<()> {
        let appointment = sqlx::query!(
            r#"
                SELECT a.patient_id, ts.doctor_id
                FROM appointments a
                JOIN time_slots ts ON ts.timeslot_id = a.timeslot_id
                WHERE a.appointment_id = $1
            "#,
            rec.appointment_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(appointment) = appointment else {
            return Err(AppError::NotFound);
        };

        if appointment.patient_id != patient_id || appointment.doctor_id != doctor_id {
            return Err(AppError::Forbidden);
        }

        let rows = sqlx::query!(
            r#"INSERT INTO diagnoses (appointment_id, patient_id, doctor_id, symptom) 
               VALUES ($1, $2, $3, $4)"#,
            rec.appointment_id,
            patient_id,
            doctor_id,
            rec.symptom
        )
        .execute(&self.pool)
        .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::Conflict);
        }
        Ok(())
    }
}
