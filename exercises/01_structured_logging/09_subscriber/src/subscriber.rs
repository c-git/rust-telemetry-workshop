use helpers::MockWriter;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::Resource;
use tonic::metadata::MetadataMap;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

pub fn init_test_subscriber() -> MockWriter {
    // JSON
    let writer = MockWriter::new();
    let writer2 = writer.clone();
    let json = tracing_subscriber::fmt::layer()
        .with_writer(move || writer.clone())
        .with_span_events(FmtSpan::NEW | FmtSpan::EXIT)
        .json()
        .flatten_event(true);

    // Honeycomb
    let honeycomb_key = helpers::honeycomb_key();
    let mut map = MetadataMap::with_capacity(1);
    map.insert("x-honeycomb-team", honeycomb_key.try_into().unwrap());
    let honeycomb = OpenTelemetryLayer::new(
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
                Resource::new(vec![KeyValue::new(
                    "service.name",
                    "rust-telemetry-workshop",
                )]),
            ))
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("https://api.honeycomb.io/api/traces")
                    .with_timeout(std::time::Duration::from_secs(5))
                    .with_metadata(map),
            )
            .install_simple()
            .unwrap(),
    );

    tracing_subscriber::registry()
        .with(json.with_filter(LevelFilter::INFO))
        .with(honeycomb.with_filter(LevelFilter::INFO))
        .init();

    writer2
}
