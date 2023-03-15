use log::info;

fn main() {
    if !cfg!(feature = "local_url") {
        panic!("Test only with local url");
    }

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    info!("Test Vaillant Rust");
}
