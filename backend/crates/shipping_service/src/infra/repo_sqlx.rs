use crate::{
    app::ShippingRepo,
    domain::{
        OrderStatus, ShippingAddressReq, ShippingAddressResp, ShippingMapPoints, ShippingOrderItem,
        ShippingOrderSummary, ShippingStatusEntry, ShippingStatusTimeline,
    },
};
use common::error::AppResult;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxShippingRepo {
    pool: PgPool,
}

impl SqlxShippingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ShippingRepo for SqlxShippingRepo {
    async fn fetch_address(&self, patient_id: Uuid) -> AppResult<Option<ShippingAddressResp>> {
        let addr = sqlx::query_as!(
            ShippingAddressResp,
            r#"
            SELECT
                first_name,
                last_name,
                address,
                postal_code,
                phone,
                lat,
                lon
            FROM shipping_address
            WHERE patient_id = $1
            "#,
            patient_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(addr)
    }

    async fn upsert_address(&self, patient_id: Uuid, req: &ShippingAddressReq) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO shipping_address (
                patient_id, first_name, last_name, address, postal_code, phone, lat, lon
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (patient_id) DO UPDATE
            SET first_name = EXCLUDED.first_name,
                last_name = EXCLUDED.last_name,
                address = EXCLUDED.address,
                postal_code = EXCLUDED.postal_code,
                phone = EXCLUDED.phone,
                lat = EXCLUDED.lat,
                lon = EXCLUDED.lon
            "#,
            patient_id,
            req.first_name,
            req.last_name,
            req.address,
            req.postal_code,
            req.phone,
            req.lat,
            req.lon
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_address(&self, patient_id: Uuid, req: &ShippingAddressReq) -> AppResult<bool> {
        let rows = sqlx::query!(
            r#"
            UPDATE shipping_address
            SET first_name = $2,
                last_name = $3,
                address = $4,
                postal_code = $5,
                phone = $6,
                lat = $7,
                lon = $8
            WHERE patient_id = $1
            "#,
            patient_id,
            req.first_name,
            req.last_name,
            req.address,
            req.postal_code,
            req.phone,
            req.lat,
            req.lon
        )
        .execute(&self.pool)
        .await?;
        Ok(rows.rows_affected() > 0)
    }

    async fn list_orders(&self, patient_id: Uuid) -> AppResult<Vec<ShippingOrderSummary>> {
        let orders = sqlx::query_as!(
            ShippingOrderRow,
            r#"
            SELECT
                order_id,
                status as "status: _",
                shipping_platform,
                image_url
            FROM orders
            WHERE patient_id = $1
            ORDER BY created_at DESC
            "#,
            patient_id
        )
        .fetch_all(&self.pool)
        .await?;

        let order_ids: Vec<i32> = orders.iter().map(|row| row.order_id).collect();
        let mut items_map: HashMap<i32, Vec<ShippingOrderItem>> = HashMap::new();
        if !order_ids.is_empty() {
            let items = sqlx::query_as!(
                ShippingItemRow,
                r#"
                SELECT
                    oi.order_id,
                    oi.medicine_id,
                    m.medicine_name,
                    oi.quantity,
                    oi.unit_price::float8 AS "unit_price!",
                    m.image_url AS "image_url?"
                FROM order_items oi
                JOIN medicines m ON m.medicine_id = oi.medicine_id
                WHERE oi.order_id = ANY($1)
                ORDER BY oi.order_id
                "#,
                &order_ids
            )
            .fetch_all(&self.pool)
            .await?;

            for item in items {
                items_map
                    .entry(item.order_id)
                    .or_default()
                    .push(ShippingOrderItem {
                        medicine_id: item.medicine_id,
                        medicine_name: item.medicine_name,
                        quantity: item.quantity,
                        unit_price: item.unit_price,
                        img_url: item.image_url,
                    });
            }
        }

        let summaries = orders
            .into_iter()
            .map(|row| ShippingOrderSummary {
                order_id: row.order_id,
                status_code: row.status.code(),
                status_label: row.status.label().to_string(),
                shipping_platform: row.shipping_platform,
                image_url: row.image_url,
                items: items_map.remove(&row.order_id).unwrap_or_default(),
            })
            .collect();
        Ok(summaries)
    }

    async fn order_timeline(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<Option<ShippingStatusTimeline>> {
        let order = sqlx::query!(
            r#"
            SELECT
                order_id,
                shipping_platform,
                image_url
            FROM orders
            WHERE order_id = $1
              AND patient_id = $2
            "#,
            order_id,
            patient_id
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(order) = order else {
            return Ok(None);
        };

        let statuses = sqlx::query_as!(
            ShippingStatusRow,
            r#"
            SELECT
                order_id AS "_order_id!",
                at,
                details,
                lat,
                lon
            FROM shipping_status
            WHERE order_id = $1
            ORDER BY at
            "#,
            order_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(ShippingStatusTimeline {
            order_id,
            shipping_platform: order.shipping_platform,
            image_url: order.image_url,
            status: statuses
                .into_iter()
                .map(ShippingStatusEntry::from)
                .collect(),
        }))
    }

    async fn map_points(
        &self,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<Option<ShippingMapPoints>> {
        let row = sqlx::query!(
            r#"
            SELECT
                sa.lat AS address_lat,
                sa.lon AS address_lon,
                status_point.lat AS shipment_lat,
                status_point.lon AS shipment_lon
            FROM orders o
            LEFT JOIN shipping_address sa ON sa.patient_id = o.patient_id
            LEFT JOIN LATERAL (
                SELECT lat, lon
                FROM shipping_status
                WHERE order_id = o.order_id
                ORDER BY at DESC
                LIMIT 1
            ) status_point ON TRUE
            WHERE o.order_id = $1
              AND o.patient_id = $2
            "#,
            order_id,
            patient_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ShippingMapPoints {
            shipment_lat: row.shipment_lat,
            shipment_lon: row.shipment_lon,
            address_lat: row.address_lat,
            address_lon: row.address_lon,
        }))
    }
}

#[derive(sqlx::FromRow)]
struct ShippingOrderRow {
    order_id: i32,
    status: OrderStatus,
    shipping_platform: Option<String>,
    image_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ShippingItemRow {
    order_id: i32,
    medicine_id: i32,
    medicine_name: String,
    quantity: i32,
    unit_price: f64,
    image_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ShippingStatusRow {
    _order_id: i32,
    at: time::OffsetDateTime,
    details: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
}

impl From<ShippingStatusRow> for ShippingStatusEntry {
    fn from(row: ShippingStatusRow) -> Self {
        Self {
            at: row.at,
            details: row.details,
            lat: row.lat,
            lon: row.lon,
        }
    }
}
