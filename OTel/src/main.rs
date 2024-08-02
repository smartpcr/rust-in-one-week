use std::task::Context;
use opentelemetry::{
    global,
    trace::{TraceError, Tracer, TracerProvider as _},
};
use opentelemetry_sdk::trace::TracerProvider;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), TraceError> {
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("example-tracer");

    // create a parent span
    tracer.in_span("main", |cx| async {
        // create a child span
        tokio::join!(
            do_work(&tracer),
            do_work(&tracer),
            do_work(&tracer)
        );
    }).await;

    global::shutdown_tracer_provider();
    Ok(())
}

async fn do_work<T: Tracer>(tracer: &T) {
    tracer.in_span("do_work", |cx| async {
        // sleep for a bit to simulate work
        sleep(Duration::from_millis(100)).await;
    }).await;
}
