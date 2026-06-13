use tracing_log::LogTracer;
use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

pub fn get_subscriber<T>(
    name: String,
    env_filter: String,
    sink: T
) -> impl Subscriber + Sync + Send
    where
        T: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{

    // env layer
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Create bunyan formatting layer
    let formatting_layer = BunyanFormattingLayer::new(
        name.into(),
        sink
    );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    subscriber
    
}

pub fn init_subscriber(
    subscriber: impl Subscriber + Send + Sync
) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");

}