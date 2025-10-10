use crate::app::AuthRepo;
use common::error::{AppError, AppResult};
use db::PgTx;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxAuthRepo {
    pool: PgPool,
}
impl SqlxAuthRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AuthRepo for SqlxAuthRepo {
    async fn create_patient(
        &self,
        hn: i32,
        citizen_id: String,
        first_name: String,
        last_name: String,
        phone: String,
        password: String,
    ) -> AppResult<Uuid> {
        let mut tx = self.pool.begin().await?;
        let user_id = sqlx::query_scalar!(
            r#"INSERT INTO users (phone, first_name, last_name, citizen_id, password)
               VALUES ($1,$2,$3,$4,$5) RETURNING user_id"#,
            phone,
            first_name,
            last_name,
            citizen_id,
            password
        )
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query!(
            r#"INSERT INTO patient_profile (user_id, hn) VALUES ($1,$2)
               ON CONFLICT (user_id) DO UPDATE SET hn = EXCLUDED.hn"#,
            user_id,
            hn
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"INSERT INTO user_roles (user_id, role) VALUES ($1,'PATIENT') ON CONFLICT DO NOTHING"#,
            user_id
        ).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(user_id)
    }

    async fn login_patient(
        &self,
        hn: i32,
        citizen_id: String,
        password: String,
    ) -> AppResult<Uuid> {
        let row = sqlx::query!(
            r#"SELECT u.user_id, u.password
               FROM users u JOIN patient_profile p ON p.user_id = u.user_id
               WHERE p.hn=$1 AND u.citizen_id=$2"#,
            hn,
            citizen_id
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(r) = row else {
            return Err(AppError::Unauthorized);
        };
        // NOTE: password is stored as plaintext in migrations; in real code hash+verify
        if r.password != password {
            return Err(AppError::Unauthorized);
        }
        Ok(r.user_id)
    }

    async fn upsert_medical_rights(
        &self,
        items: Vec<(i32, String, String, String)>,
    ) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        for (mr_id, name, details, image_url) in items {
            sqlx::query!(
                r#"INSERT INTO medical_rights (mr_id, name, details, img_url)
                   OVERRIDING SYSTEM VALUE VALUES ($1,$2,$3,$4)
                   ON CONFLICT (mr_id) DO UPDATE SET name=EXCLUDED.name, details=EXCLUDED.details, img_url=EXCLUDED.img_url"#,
                mr_id, name, details, image_url
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn create_doctor(
        &self,
        mln: String,
        citizen_id: String,
        first_name: String,
        last_name: String,
        phone: String,
        password: String,
    ) -> AppResult<Uuid> {
        let mut tx = self.pool.begin().await?;
        let user_id = sqlx::query_scalar!(
            r#"INSERT INTO users (phone, first_name, last_name, citizen_id, password)
               VALUES ($1,$2,$3,$4,$5) RETURNING user_id"#,
            phone,
            first_name,
            last_name,
            citizen_id,
            password
        )
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query!(
            r#"INSERT INTO doctor_profile (user_id, mln) VALUES ($1,$2)
               ON CONFLICT (user_id) DO UPDATE SET mln = EXCLUDED.mln"#,
            user_id,
            mln
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"INSERT INTO user_roles (user_id, role) VALUES ($1,'DOCTOR') ON CONFLICT DO NOTHING"#,
            user_id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(user_id)
    }

    async fn login_doctor(
        &self,
        mln: String,
        citizen_id: String,
        password: String,
    ) -> AppResult<Uuid> {
        let row = sqlx::query!(
            r#"SELECT u.user_id, u.password
               FROM users u JOIN doctor_profile d ON d.user_id = u.user_id
               WHERE d.mln=$1 AND u.citizen_id=$2"#,
            mln,
            citizen_id
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(r) = row else {
            return Err(AppError::Unauthorized);
        };
        if r.password != password {
            return Err(AppError::Unauthorized);
        }
        Ok(r.user_id)
    }
}
