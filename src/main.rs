use axum::{
    routing::get,
    response::Response,
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    trace::{
        TraceLayer,
        DefaultOnResponse
    }
};
use tracing::Level;
use tracing_appender::rolling::daily;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    filter::{
        EnvFilter,
        LevelFilter
    },
    Layer
};
use std::time::Duration;
use axum::body::BoxBody;
use tracing::Span;
// use http::{Response};
// use hyper::body::Body;
// use tracing_subscriber::fmt::writer::MakeWriterExt;


#[tokio::main]
async fn main() {

    let axum_test_log_filter = EnvFilter::try_from_env("AXUM_TEST_LOG")
        .unwrap_or_else(|err| -> EnvFilter {
            println!("Something goes wrong with \"AXUM_TEST_LOG\" env var: {:?}", err);
            println!("Enable \"RUST_LOG\" directives if available, ERROR level otherwise");

            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy()
        });

    let log_file = daily("./logs", "axum_test");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(log_file)
                .with_target(false)
                .with_ansi(false)
                .compact()
                .with_filter(axum_test_log_filter)
        )
        .init();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        // .layer(
        //     TraceLayer::new_for_http()
        //         // .make_span_with(trace::DefaultMakeSpan::new()
        //         //     .level(Level::INFO))
        //         // .on_response(|response: &Response<Body>, latency: Duration, _span: &Span| {
        //         //     tracing::debug!("response generated in {:?}", latency)
        //         // })
        //         .on_response(DefaultOnResponse::new()
        //             .level(Level::INFO)
        //             .latency_unit(LatencyUnit::Micros))
        // )
        // .layer(prometheus_layer)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .on_response(|response: &Response<BoxBody>, latency: Duration, _span: &Span| {
                            tracing::info!("response generated in {:?}", latency)
                        })
                        // .on_response(DefaultOnResponse::new()
                        //     .level(Level::DEBUG)
                        //     .latency_unit(LatencyUnit::Micros))
                )
                .layer(prometheus_layer)
        )
        ;

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

