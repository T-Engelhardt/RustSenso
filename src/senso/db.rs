use crate::response;

#[derive(Debug, PartialEq)]
pub struct SensorData {
    outdoor_temp: Option<f64>,                        //system status
    domestic_hot_water_tank_temperature: Option<f64>, //live report; Device ID: Control_DHW => Report ID: DomesticHotWaterTankTemperature
    water_pressure_sensor: Option<f64>, //live report; Device ID: Control_SYS_senso => Report ID: WaterPressureSensor
    flow_temperature_sensor: Option<f64>, //live report; Device ID: Control_CC1 => Report ID: FlowTemperatureSensor
}

impl SensorData {
    pub fn new(
        status: &Result<response::status::Root, ()>,
        live_report: &Result<response::live_report::Root, ()>,
    ) -> SensorData {
        let mut domestic_hot_water_tank_temperature = None;
        let mut water_pressure_sensor = None;
        let mut flow_temperature_sensor = None;

        if let Ok(data) = live_report {
            domestic_hot_water_tank_temperature = data
                .body
                .find_report_for_device("Control_DHW", "DomesticHotWaterTankTemperature")
                .map(|r| r.value);
            water_pressure_sensor = data
                .body
                .find_report_for_device("Control_SYS_senso", "WaterPressureSensor")
                .map(|r| r.value);
            flow_temperature_sensor = data
                .body
                .find_report_for_device("Control_CC1", "FlowTemperatureSensor")
                .map(|r| r.value);
        }

        SensorData {
            outdoor_temp: if let Ok(data) = status {
                Some(data.body.outside_temperature)
            } else {
                None
            },
            domestic_hot_water_tank_temperature,
            water_pressure_sensor,
            flow_temperature_sensor,
        }
    }

    pub fn new_raw(
        outdoor_temp: Option<f64>,
        domestic_hot_water_tank_temperature: Option<f64>,
        water_pressure_sensor: Option<f64>,
        flow_temperature_sensor: Option<f64>,
    ) -> SensorData {
        SensorData {
            outdoor_temp,
            domestic_hot_water_tank_temperature,
            water_pressure_sensor,
            flow_temperature_sensor,
        }
    }
}
