//! Captain: a native desktop client for Docker.

mod actions;
mod connect;
mod window;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "captain=info,captain_docker=info".into()),
        )
        .init();

    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            actions::register(cx);
            window::open(cx);
            cx.activate(true);
        });
}
