pub mod config;
pub mod error;
pub mod handlers;

use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
};

use handlers::{AppState, echo_handler};

pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new().fallback(echo_handler);

    // Add Prometheus metrics endpoint if enabled
    #[cfg(feature = "prometheus")]
    if state.config.prometheus {
        router = router.route("/metrics", get(handlers::metrics_handler));
    }

    router = router.layer(
        ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(state.config.max_body_size))
            .layer(CompressionLayer::new())
            .layer(TraceLayer::new_for_http()),
    );

    // Add CORS if enabled
    if state.config.enable_cors {
        router = router.layer(CorsLayer::permissive());
    }

    router.with_state(state)
}
