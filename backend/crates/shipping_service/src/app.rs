use crate::domain::{
    OrderStatus, ShippingAddressReq, ShippingAddressResp, ShippingMapPoints, ShippingOrderSummary,
    ShippingStatusTimeline,
};
use common::error::{AppError, AppResult};
use uuid::Uuid;

#[expect(async_fn_in_trait)]
pub trait ShippingRepo: Send + Sync {
    async fn fetch_address(&self, patient_id: Uuid) -> AppResult<Option<ShippingAddressResp>>;
    async fn upsert_address(&self, patient_id: Uuid, req: &ShippingAddressReq) -> AppResult<()>;
    async fn update_address(&self, patient_id: Uuid, req: &ShippingAddressReq) -> AppResult<bool>;
    async fn list_orders(&self, patient_id: Uuid) -> AppResult<Vec<ShippingOrderSummary>>;
    async fn order_timeline(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<Option<ShippingStatusTimeline>>;
    async fn map_points(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<Option<ShippingMapPoints>>;
}

#[derive(Clone)]
pub struct ShippingService<R: ShippingRepo> {
    pub repo: R,
}

impl<R: ShippingRepo> ShippingService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn address(&self, patient_id: Uuid) -> AppResult<Option<ShippingAddressResp>> {
        self.repo.fetch_address(patient_id).await
    }

    pub async fn upsert_address(
        &self,
        patient_id: Uuid,
        req: &ShippingAddressReq,
    ) -> AppResult<()> {
        self.repo.upsert_address(patient_id, req).await
    }

    pub async fn update_address(
        &self,
        patient_id: Uuid,
        req: &ShippingAddressReq,
    ) -> AppResult<()> {
        let updated = self.repo.update_address(patient_id, req).await?;
        if updated {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn list_orders(
        &self,
        patient_id: Uuid,
        filter: Option<OrderStatus>,
    ) -> AppResult<Vec<ShippingOrderSummary>> {
        let orders = self.repo.list_orders(patient_id).await?;
        let filtered = orders
            .into_iter()
            .filter(|order| {
                filter
                    .map(|f| order.status_code == f.code())
                    .unwrap_or(true)
            })
            .collect();
        Ok(filtered)
    }

    pub async fn order_timeline(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<ShippingStatusTimeline> {
        match self.repo.order_timeline(patient_id, order_id).await? {
            Some(t) => Ok(t),
            None => Err(AppError::NotFound),
        }
    }

    pub async fn map_points(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<ShippingMapPoints> {
        match self.repo.map_points(patient_id, order_id).await? {
            Some(points) => Ok(points),
            None => Err(AppError::NotFound),
        }
    }
}
