use crate::domain::{CreateOrderItemReq, CreateOrderReq, OrderDetail};
use common::error::{AppError, AppResult};
use db::PgTx;
use uuid::Uuid;

#[expect(async_fn_in_trait)]
pub trait OrderRepo: Send + Sync {
    async fn list_orders(&self, patient_id: Uuid) -> AppResult<Vec<OrderDetail>>;
    async fn create_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        req: &CreateOrderReq,
    ) -> AppResult<i32>;
    async fn insert_order_item(
        &self,
        tx: &mut PgTx<'_>,
        order_id: i32,
        item: &CreateOrderItemReq,
    ) -> AppResult<()>;
    async fn confirm_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct OrderService<R: OrderRepo> {
    pub repo: R,
}

impl<R: OrderRepo> OrderService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_orders(&self, patient_id: Uuid) -> AppResult<Vec<OrderDetail>> {
        self.repo.list_orders(patient_id).await
    }

    pub async fn create_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        req: &CreateOrderReq,
    ) -> AppResult<i32> {
        let order_id = self.repo.create_order(tx, patient_id, req).await?;
        for item in &req.items {
            self.repo.insert_order_item(tx, order_id, item).await?;
        }
        Ok(order_id)
    }

    pub async fn confirm_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<()> {
        if self.repo.confirm_order(tx, patient_id, order_id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}
