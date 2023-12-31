use anyhow::anyhow;
use log::{debug, info};
use rusqlite::{params, Connection};

use crate::{response, yp::YpData};

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

pub struct DB {
    conn: Connection,
}

impl DB {
    // Opens sqlite at PATH or if None in memory
    pub fn new(path: Option<&str>) -> Result<DB, anyhow::Error> {
        let conn: Connection;

        if let Some(p) = path {
            conn = rusqlite::Connection::open(p)?;
            info!("Opening Sqlite DB at {}.", p);
        } else {
            conn = rusqlite::Connection::open_in_memory()?;
            info!("Opening Sqlite DB in memory.");
        }

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS Temperature (
            id INTEGER PRIMARY KEY,
            time INTEGER NOT NULL,
            outdoor REAL,
            hotwatertank REAL,
            waterpressure REAL,
            heatingcircuit REAL)"#,
            (),
        )?;

        conn.execute(
            r#" CREATE TABLE IF NOT EXISTS Usage (
                id INTEGER PRIMARY KEY,
                time INTEGER NOT NULL UNIQUE,
                ch_hp_y INTEGER,
                ch_hp_p INTEGER,
                ch_bo_p INTEGER,
                ch_yp REAL,
                hw_hp_y INTEGER,
                hw_hp_p INTEGER,
                hw_bo_p INTEGER,
                hw_yp REAL,
                total_y INTEGER,
                total_p INTEGER,
                total_yp REAL)"#,
            (),
        )?;

        Ok(DB { conn })
    }

    pub fn insert_sensor_data(&self, sensor_data: SensorData) -> Result<(), anyhow::Error> {
        self.conn.execute(
            r#"INSERT INTO Temperature (id, time, outdoor, hotwatertank, waterpressure, heatingcircuit)
            VALUES (NULL, STRFTIME('%s'), ?1, ?2, ?3, ?4)"#,
         (sensor_data.outdoor_temp,
            sensor_data.domestic_hot_water_tank_temperature,
            sensor_data.water_pressure_sensor,
            sensor_data.flow_temperature_sensor))?;

        info!("Inserted Sensor Data into DB");
        Ok(())
    }

    pub fn get_sensor_data(&self, id: usize) -> Result<SensorData, anyhow::Error> {
        let mut stmt = self.conn.prepare("SELECT outdoor, hotwatertank, waterpressure, heatingcircuit FROM Temperature WHERE id = :id;")?;

        let mut data_iter = stmt.query_map(params![id], |row| {
            Ok(SensorData::new_raw(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        if let Some(data) = data_iter.next() {
            debug!("Found Sensor Data at id: {}.", id);
            data.map_err(|e| anyhow!(e))
        } else {
            Err(anyhow!("No SensorData found with for id."))
        }
    }

    pub fn insert_yp_data(&self, yp_data: &YpData) -> Result<(), anyhow::Error> {
        self.conn.execute(
            r#"INSERT OR REPLACE INTO Usage (id, time, ch_hp_y, ch_hp_p, ch_bo_p, ch_yp, hw_hp_y, hw_hp_p, hw_bo_p, hw_yp, total_y, total_p, total_yp)
            VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
         (yp_data.ts.timestamp(),
            yp_data.ch_hp_y,
            yp_data.ch_hp_p,
            yp_data.ch_bo_p,
            yp_data.ch_yp,
            yp_data.hw_hp_y,
            yp_data.hw_hp_p,
            yp_data.hw_bo_p,
            yp_data.hw_yp,
            yp_data.total_y,
            yp_data.total_p,
            yp_data.total_yp))?;

        info!("Inserted YP Data into DB for day: {}", yp_data.ts);
        Ok(())
    }
}
