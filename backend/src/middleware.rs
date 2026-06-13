use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tower_http::cors::{Any, CorsLayer};

/// Initialise the tracing subscriber with env-filter support.
///
/// Respects the `RUST_LOG` environment variable; defaults to
/// `backend=info` if unset.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info".into()),
        )
        .init();
}

/// Return a CORS layer that permits requests from the Vite dev server.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Axum middleware that guarantees every response carries an
/// `X-Request-Id` header.
///
/// - If the client sends `X-Request-Id` it is passed through.
/// - Otherwise a fresh UUID v4 is generated.
/// - The id is attached to a tracing span so that every log emitted
///   while handling the request includes the request id.
pub async fn request_id(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let span = tracing::info_span!("request", %request_id);
    let _guard = span.enter();

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("X-Request-Id", HeaderValue::from_str(&request_id).unwrap());
    response
}
