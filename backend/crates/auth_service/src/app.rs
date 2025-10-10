use common::error::AppResult;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService<R: AuthRepo> {
    pub repo: R,
}

impl<R: AuthRepo> AuthService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait AuthRepo: Send + Sync {
    #[expect(async_fn_in_trait)]
    async fn create_patient(
        &self,
        hn: i32,
        citizen_id: String,
        first_name: String,
        last_name: String,
        phone: String,
        password: String,
    ) -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn login_patient(&self, hn: i32, citizen_id: String, password: String)
    -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn upsert_medical_rights(
        &self,
        items: Vec<(i32, String, String, String)>,
    ) -> AppResult<()>;
    #[expect(async_fn_in_trait)]
    async fn create_doctor(
        &self,
        mln: String,
        citizen_id: String,
        first_name: String,
        last_name: String,
        phone: String,
        password: String,
    ) -> AppResult<Uuid>;
    #[expect(async_fn_in_trait)]
    async fn login_doctor(
        &self,
        mln: String,
        citizen_id: String,
        password: String,
    ) -> AppResult<Uuid>;
}
