/// 模拟支付网关（支付宝/微信支付）
pub struct MockPaymentGateway;

impl MockPaymentGateway {
    pub async fn create_payment(&self, plan: &str, amount_cents: u64) -> String {
        tracing::info!(plan = %plan, amount = amount_cents, "模拟创建支付订单");
        format!("https://mock-pay.tsangwu.com/pay?plan={}&amount={}", plan, amount_cents)
    }

    pub async fn verify_callback(&self, _signature: &str, _body: &str) -> bool {
        tracing::info!("模拟验证支付回调签名");
        true
    }

    pub async fn query_order(&self, order_id: &str) -> serde_json::Value {
        tracing::info!(order_id = %order_id, "模拟查询支付订单");
        serde_json::json!({
            "order_id": order_id,
            "status": "paid",
            "paid_at": chrono::Utc::now().to_rfc3339(),
        })
    }
}
