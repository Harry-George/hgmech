use ui::App;
use tracing_subscriber::{filter::Targets, prelude::*};
use tracing_web::MakeWebConsoleWriter;

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .without_time()
        .with_writer(MakeWebConsoleWriter::new())
        .with_filter(
            Targets::new()
                .with_target("leptos", tracing::Level::DEBUG)
                .with_default(tracing::Level::TRACE),
        );

    tracing_subscriber::registry().with(fmt_layer).init();
    console_error_panic_hook::set_once();

    tracing::info!("Starting Leptos application");
    leptos::mount::mount_to_body(App);
}
