use super::super::app::PrescriptionRepo;
use super::super::domain::*;
use common::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxPrescriptionRepo {
    pool: PgPool,
}
impl SqlxPrescriptionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PrescriptionRepo for SqlxPrescriptionRepo {
    async fn by_id(&self, id: Uuid) -> AppResult<Vec<Prescription>> {
        let recs = sqlx::query_as!(
            Prescription,
            r#"
            SELECT 
                a.prescription_id,
                a.patient_id,
                b.medicine_name,
                a.dosage,
                a.amount,
                a.on_going,
                a.doctor_comment AS "doctor_comment?",
                b.image_url AS "image_url?"
            FROM prescriptions a 
            JOIN medicines b ON a.medicine_id = b.medicine_id
            WHERE a.patient_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn search_medicines_by_name(&self, input: &str) -> AppResult<Vec<(i32, String)>> {
        let q = format!("%{}%", input);
        let rows = sqlx::query!(
            r#"SELECT medicine_id, medicine_name FROM medicines WHERE medicine_name ILIKE $1 ORDER BY medicine_name LIMIT 50"#,
            q
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.medicine_id, r.medicine_name))
            .collect())
    }

    async fn get_medicine_info(
        &self,
        medicine_id: i32,
    ) -> AppResult<Option<(i32, String, String)>> {
        let row = sqlx::query!(
            r#"SELECT medicine_id, medicine_name, image_url FROM medicines WHERE medicine_id = $1"#,
            medicine_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            (
                r.medicine_id,
                r.medicine_name,
                r.image_url.unwrap_or_default(),
            )
        }))
    }

    async fn create_prescription(&self, input: CreatePrescriptionInput) -> AppResult<i32> {
        let CreatePrescriptionInput {
            patient_id,
            medicine_id,
            dosage,
            amount,
            on_going,
            doctor_comment,
        } = input;
        let rec = sqlx::query!(
            r#"INSERT INTO prescriptions (patient_id, medicine_id, dosage, amount, on_going, doctor_comment)
               VALUES ($1,$2,$3,$4,$5,$6)
               RETURNING prescription_id"#,
            patient_id, medicine_id, dosage, amount, on_going, doctor_comment
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.prescription_id)
    }

    async fn update_prescription(&self, input: UpdatePrescriptionInput) -> AppResult<()> {
        let UpdatePrescriptionInput {
            prescription_id,
            medicine_id,
            patient_id,
            dosage,
            amount,
            on_going,
            doctor_comment,
        } = input;
        let rows = sqlx::query!(
            r#"UPDATE prescriptions
               SET patient_id=$1, medicine_id=$2, dosage=$3, amount=$4, on_going=$5, doctor_comment=$6
               WHERE prescription_id=$7"#,
            patient_id, medicine_id, dosage, amount, on_going, doctor_comment, prescription_id
        )
        .execute(&self.pool)
        .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    async fn delete_prescription(&self, prescription_id: i32) -> AppResult<()> {
        let rows = sqlx::query!(
            r#"DELETE FROM prescriptions WHERE prescription_id = $1"#,
            prescription_id
        )
        .execute(&self.pool)
        .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
