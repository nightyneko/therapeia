use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
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
pub struct ShippingAddressResp {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub postal_code: String,
    pub phone: String,
    #[schema(nullable = true)]
    pub lat: Option<f64>,
    #[schema(nullable = true)]
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ShippingAddressReq {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub postal_code: String,
    pub phone: String,
    #[schema(nullable = true)]
    pub lat: Option<f64>,
    #[schema(nullable = true)]
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShippingOrderItem {
    pub medicine_id: i32,
    pub medicine_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    #[schema(nullable = true)]
    pub img_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShippingOrderSummary {
    pub order_id: i32,
    pub status_code: i32,
    pub status_label: String,
    #[schema(nullable = true)]
    pub shipping_platform: Option<String>,
    #[schema(nullable = true)]
    pub image_url: Option<String>,
    pub items: Vec<ShippingOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShippingStatusEntry {
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub at: OffsetDateTime,
    #[schema(nullable = true)]
    pub details: Option<String>,
    #[schema(nullable = true)]
    pub lat: Option<f64>,
    #[schema(nullable = true)]
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShippingStatusTimeline {
    pub order_id: i32,
    #[schema(nullable = true)]
    pub shipping_platform: Option<String>,
    #[schema(nullable = true)]
    pub image_url: Option<String>,
    pub status: Vec<ShippingStatusEntry>,
}

#[derive(Debug, Clone)]
pub struct ShippingMapPoints {
    pub shipment_lat: Option<f64>,
    pub shipment_lon: Option<f64>,
    pub address_lat: Option<f64>,
    pub address_lon: Option<f64>,
}

impl ShippingMapPoints {
    pub fn shipment_coordinates(&self) -> Option<(f64, f64)> {
        match (self.shipment_lat, self.shipment_lon) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }

    pub fn address_coordinates(&self) -> Option<(f64, f64)> {
        match (self.address_lat, self.address_lon) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }

    pub fn has_required_coordinates(&self) -> bool {
        self.shipment_coordinates().is_some() && self.address_coordinates().is_some()
    }
}
