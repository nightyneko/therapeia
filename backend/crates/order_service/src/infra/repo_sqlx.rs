use crate::{
    app::OrderRepo,
    domain::{CreateOrderItemReq, CreateOrderReq, OrderDetail, OrderItemSummary, OrderStatus},
};
use common::error::AppResult;
use db::PgTx;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqlxOrderRepo {
    pool: PgPool,
}

impl SqlxOrderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OrderRepo for SqlxOrderRepo {
    async fn list_orders(&self, patient_id: Uuid) -> AppResult<Vec<OrderDetail>> {
        let orders = sqlx::query_as!(
            OrderRow,
            r#"
            SELECT
                order_id,
                status as "status: _",
                shipping_platform,
                payment_platform
            FROM orders
            WHERE patient_id = $1
            ORDER BY created_at DESC
            "#,
            patient_id
        )
        .fetch_all(&self.pool)
        .await?;

        let order_ids: Vec<i32> = orders.iter().map(|o| o.order_id).collect();
        let mut items_map: HashMap<i32, Vec<OrderItemSummary>> = HashMap::new();

        if !order_ids.is_empty() {
            let items = sqlx::query_as!(
                OrderItemRow,
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
                    .push(OrderItemSummary {
                        medicine_id: item.medicine_id,
                        amount: item.quantity,
                        price: item.unit_price,
                        medicine_name: item.medicine_name,
                        img_link: item.image_url,
                    });
            }
        }

        let details = orders
            .into_iter()
            .map(|order| {
                let items = items_map.remove(&order.order_id).unwrap_or_default();
                let tot_price = items
                    .iter()
                    .map(|item| item.price * item.amount as f64)
                    .sum();
                OrderDetail {
                    order_id: order.order_id,
                    status_code: order.status.code(),
                    status_label: order.status.label().to_string(),
                    shipping_platform: order.shipping_platform,
                    payment_platform: order.payment_platform,
                    tot_price,
                    items,
                }
            })
            .collect();
        Ok(details)
    }

    async fn create_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        req: &CreateOrderReq,
    ) -> AppResult<i32> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO orders (patient_id, shipping_platform, payment_platform, image_url)
            VALUES ($1, $2, $3, $4)
            RETURNING order_id
            "#,
            patient_id,
            req.shipping_platform,
            req.payment_platform,
            req.image_url
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(rec.order_id)
    }

    async fn insert_order_item(
        &self,
        tx: &mut PgTx<'_>,
        order_id: i32,
        item: &CreateOrderItemReq,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO order_items (order_id, medicine_id, quantity, unit_price)
            VALUES ($1, $2, $3, $4::float8)
            "#,
            order_id,
            item.medicine_id,
            item.amount,
            item.price
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn confirm_order(
        &self,
        tx: &mut PgTx<'_>,
        patient_id: Uuid,
        order_id: i32,
    ) -> AppResult<bool> {
        let rows = sqlx::query!(
            r#"
            UPDATE orders
            SET status = 'SHIPPING'
            WHERE order_id = $1
              AND patient_id = $2
              AND status = 'PENDING'
            "#,
            order_id,
            patient_id
        )
        .execute(&mut **tx)
        .await?;
        Ok(rows.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct OrderRow {
    order_id: i32,
    status: OrderStatus,
    shipping_platform: Option<String>,
    payment_platform: Option<String>,
}

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    order_id: i32,
    medicine_id: i32,
    medicine_name: String,
    quantity: i32,
    unit_price: f64,
    image_url: Option<String>,
}
