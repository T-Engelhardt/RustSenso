use log::info;
use senso::urls;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    info!("Test Vaillant Rust");

    println!(
        "{:#?}",
        urls::VaillantV4::new(urls::UrlBase::VaillantAPI, "".into())
    );
}
