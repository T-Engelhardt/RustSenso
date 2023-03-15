use env_logger::Env;
use log::{debug, error, info};
use senso::{connector::Connector, db::SensorData};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut c = Connector::new("21223900202609620938071939N6".into());
    c.login("T.Engelhardt", "vZW5Sz4Xmj#I")
        .map_err(|e| error!("{}", e.to_string()))
        .unwrap();

    let status = c.system_status().map_err(|e| error!("{}", e.to_string()));
    debug!("{:#?}", status);

    let live_report = c.live_report().map_err(|e| error!("{}", e.to_string()));
    debug!("{:#?}", live_report);

    let data = SensorData::new(&status, &live_report);

    info!("{:#?}", data);
}
