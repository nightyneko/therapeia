use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    PENDING,
    SHIPPING,
    CANCELED,
    SUCCESS,
}

impl OrderStatus {
    pub fn code(self) -> i32 {
        match self {
            OrderStatus::PENDING => 1,
            OrderStatus::SHIPPING => 2,
            OrderStatus::CANCELED => 3,
            OrderStatus::SUCCESS => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OrderStatus::PENDING => "PENDING",
            OrderStatus::SHIPPING => "SHIPPING",
            OrderStatus::CANCELED => "CANCELED",
            OrderStatus::SUCCESS => "SUCCESS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderItemSummary {
    pub medicine_id: i32,
    pub amount: i32,
    pub price: f64,
    pub medicine_name: String,
    #[schema(nullable = true)]
    pub img_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderDetail {
    pub order_id: i32,
    pub status_code: i32,
    pub status_label: String,
    #[schema(nullable = true)]
    pub shipping_platform: Option<String>,
    #[schema(nullable = true)]
    pub payment_platform: Option<String>,
    pub tot_price: f64,
    pub items: Vec<OrderItemSummary>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateOrderItemReq {
    pub medicine_id: i32,
    #[schema(example = 1)]
    pub amount: i32,
    #[schema(example = 120.0)]
    pub price: f64,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateOrderReq {
    pub shipping_platform: String,
    pub payment_platform: String,
    #[schema(nullable = true)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub items: Vec<CreateOrderItemReq>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrderResp {
    pub order_id: i32,
}
