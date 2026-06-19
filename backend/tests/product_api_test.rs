mod support;

use reqwest::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use std::net::SocketAddr;
use support::db;
use support::http;

async fn setup() -> (PgPool, SocketAddr) {
    let pool = db::setup_test_db().await;
    let app = http::test_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (pool, addr)
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{}{}", addr, path)
}

#[tokio::test]
async fn product_crud_and_validation() {
    let (_pool, addr) = setup().await;

    // 1. List (empty)
    let resp = reqwest::get(url(addr, "/api/v1/products")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 0);

    // 2. Create
    let resp = reqwest::Client::new()
        .post(url(addr, "/api/v1/products"))
        .json(&serde_json::json!({
            "sku_code": "TST-001",
            "name": "Test Product",
            "unit": "pcs",
            "spec": "Spec A",
            "barcode": "BAR001"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], 0);
    let product_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["sku_code"], "TST-001");
    assert_eq!(body["data"]["status"], "active");

    // 3. List (1 row)
    let resp = reqwest::get(url(addr, "/api/v1/products")).await.unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 1);

    // 4. Get by id
    let resp = reqwest::get(url(addr, &format!("/api/v1/products/{}", product_id)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["name"], "Test Product");

    // 5. Update
    let resp = reqwest::Client::new()
        .put(url(addr, &format!("/api/v1/products/{}", product_id)))
        .json(&serde_json::json!({
            "sku_code": "TST-001",
            "name": "Updated Product",
            "unit": "box",
            "spec": "Spec B",
            "barcode": "BAR002"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["name"], "Updated Product");
    assert_eq!(body["data"]["unit"], "box");

    // 6. Duplicate SKU → 409
    let resp = reqwest::Client::new()
        .post(url(addr, "/api/v1/products"))
        .json(&serde_json::json!({
            "sku_code": "TST-001",
            "name": "Dup",
            "unit": "pcs"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], 40901);

    // 7. Disable
    let resp = reqwest::Client::new()
        .patch(url(
            addr,
            &format!("/api/v1/products/{}/status", product_id),
        ))
        .json(&serde_json::json!({"status": "disabled"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["status"], "disabled");

    // 8. Filter by status
    let resp = reqwest::get(url(addr, &format!("/api/v1/products?status=disabled")))
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert!(
        body["data"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|it| it["status"] == "disabled")
    );

    // 9. Keyword search
    let resp = reqwest::get(url(addr, "/api/v1/products?keyword=Updated"))
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"]["items"].as_array().unwrap().len() >= 1);

    // 10. Not found
    let resp = reqwest::get(url(
        addr,
        "/api/v1/products/00000000-0000-0000-0000-000000000000",
    ))
    .await
    .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
