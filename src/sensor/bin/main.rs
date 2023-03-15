use env_logger::Env;
use log::{debug, error, info};
use senso::connector::Connector;

#[derive(Debug)]
struct SensorData {
    outdoor_temp: f64,                        //system status
    domestic_hot_water_tank_temperature: f64, //live report; Device ID: Control_DHW => Report ID: DomesticHotWaterTankTemperature
    water_pressure_sensor: f64, //live report; Device ID: Control_SYS_senso => Report ID: WaterPressureSensor
    flow_temperature_sensor: f64, //live report; Device ID: Control_CC1 => Report ID: FlowTemperatureSensor
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut c = Connector::new("21223900202609620938071939N6".into());
    c.login("T.Engelhardt", "vZW5Sz4Xmj#I")
        .map_err(|e| error!("{}", e.to_string()))
        .unwrap();

    let status = c
        .system_status()
        .map_err(|e| error!("{}", e.to_string()))
        .unwrap();
    debug!("{:#?}", status);

    let live_report = c
        .live_report()
        .map_err(|e| error!("{}", e.to_string()))
        .unwrap();
    debug!("{:#?}", status);

    let data = SensorData {
        outdoor_temp: status.body.outside_temperature,
        domestic_hot_water_tank_temperature: live_report
            .body
            .find_report_for_device("Control_DHW", "DomesticHotWaterTankTemperature")
            .unwrap()
            .value,
        water_pressure_sensor: live_report
            .body
            .find_report_for_device("Control_SYS_senso", "WaterPressureSensor")
            .unwrap()
            .value,
        flow_temperature_sensor: live_report
            .body
            .find_report_for_device("Control_CC1", "FlowTemperatureSensor")
            .unwrap()
            .value,
    };

    info!("{:#?}", data);
}
