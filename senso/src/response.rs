#![allow(dead_code)]
use serde::Deserialize;

// https://transform.tools/json-to-rust-serde
// Property Name Format snake_case

// META
pub mod meta {
    use super::*;

    use chrono::{DateTime, Local};

    #[derive(Debug, Deserialize)]
    pub struct MetaEmpty {}

    #[derive(Debug, Deserialize)]
    pub struct Meta {
        #[serde(rename = "resourceState")]
        pub resource_state: Vec<ResourceState>,
    }

    #[derive(Debug, Deserialize)]
    pub struct ResourceState {
        pub link: Link,
        pub state: State,
        #[serde(with = "timestamp_seconds_milli_or_not")]
        pub timestamp: DateTime<Local>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Link {
        pub rel: Rel,
        #[serde(rename = "resourceLink")]
        pub resource_link: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum State {
        Outdated,
        Synced,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub enum Rel {
        #[serde(rename = "child")]
        Child,
        #[serde(rename = "self")]
        _Self,
    }
}

// TOKEN
pub mod token {
    use super::{meta::MetaEmpty, *};

    #[derive(Debug, Deserialize)]
    pub struct Root {
        pub body: Body,
        pub meta: MetaEmpty,
    }

    #[derive(Debug, Deserialize)]
    pub struct Body {
        #[serde(rename = "authToken")]
        pub auth_token: String,
    }
}

// Status
pub mod status {
    use super::{meta::Meta, *};
    use iso8601_timestamp::Timestamp;

    #[derive(Debug, Deserialize)]
    pub struct Root {
        pub body: Body,
        pub meta: Meta,
    }

    #[derive(Debug, Deserialize)]
    pub struct Body {
        pub datetime: Timestamp,
        pub outside_temperature: f64,
    }
}

// Live report
pub mod live_report {
    use super::{meta::Meta, *};

    #[derive(Debug, Deserialize)]
    pub struct Root {
        pub body: Body,
        pub meta: Meta,
    }

    #[derive(Debug, Deserialize)]
    pub struct Body {
        pub devices: Vec<Device>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Device {
        #[serde(rename = "_id")]
        pub id: String,
        pub name: String,
        pub reports: Vec<Report>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Report {
        #[serde(rename = "_id")]
        pub id: String,
        pub name: String,
        pub value: f64,
        pub unit: String,
        pub measurement_category: MeasurementCategory,
        pub associated_device_function: Option<AssociatedDeviceFunction>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum MeasurementCategory {
        Temperature,
        Pressure,
        #[serde(rename = "AIR_QUALITY")]
        AirQuality,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum AssociatedDeviceFunction {
        Dhw,
        Heating,
        #[serde(rename = "RELATIVE_HUMIDITY")]
        RelativeHumidity,
    }

    impl Body {
        // find all reports for given device id
        pub fn find_reports_for_device(&self, device_id: &str) -> Option<&Vec<Report>> {
            self.devices
                .iter()
                .find(|d| d.id == device_id)
                .map(|x| &x.reports)
        }

        // find report for given device and report id
        pub fn find_report_for_device<'a>(
            &'a self,
            device_id: &'a str,
            report_id: &'a str,
        ) -> Option<&Report> {
            let reports = self.find_reports_for_device(device_id)?;
            find_report(reports, report_id)
        }
    }

    // find report in Vec of reports
    pub fn find_report<'a>(reports: &'a [Report], report_id: &'a str) -> Option<&'a Report> {
        reports.iter().find(|r| r.id == report_id)
    }
}

pub mod emf_devices {
    use super::{meta::MetaEmpty, *};

    use iso8601_timestamp::Timestamp;
    use strum_macros::AsRefStr;

    #[derive(Debug, Deserialize)]
    pub struct Root {
        pub body: Vec<Body>,
        pub meta: MetaEmpty,
    }

    #[derive(Debug, Deserialize)]
    pub struct Body {
        pub id: String,
        #[serde(rename = "marketingName")]
        pub marketing_name: String,
        pub reports: Vec<Report>,
        #[serde(rename = "type")]
        pub type_field: EmfDevice,
    }

    #[derive(Debug, Deserialize)]
    pub struct Report {
        #[serde(rename = "currentMeterReading")]
        pub current_meter_reading: f64,
        #[serde(rename = "energyType")]
        pub energy_type: EnergyType,
        pub from: Timestamp,
        pub function: EmfFunction,
        pub to: Timestamp,
    }

    #[derive(Debug, PartialEq, Deserialize, AsRefStr)]
    pub enum EnergyType {
        #[serde(rename = "ENVIRONMENTAL_YIELD")]
        #[strum(serialize = "ENVIRONMENTAL_YIELD")]
        EnvironmentalYield,
        #[serde(rename = "CONSUMED_ELECTRICAL_POWER")]
        #[strum(serialize = "CONSUMED_ELECTRICAL_POWER")]
        ConsumedElectricalPower,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Deserialize, AsRefStr)]
    pub enum EmfFunction {
        #[serde(rename = "DHW")]
        #[strum(serialize = "DHW")]
        DomesticHotWater,
        #[serde(rename = "CENTRAL_HEATING")]
        #[strum(serialize = "CENTRAL_HEATING")]
        CentralHeating,
        #[serde(rename = "COOLING")]
        #[strum(serialize = "COOLING")]
        Cooling,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
    pub enum EmfDevice {
        #[serde(rename = "BOILER")]
        Boiler,
        #[serde(rename = "HEAT_PUMP")]
        HeatPump,
    }
}

pub mod emf_report_device {
    use super::{meta::MetaEmpty, *};
    use iso8601_timestamp::Timestamp;

    #[derive(Debug, Deserialize)]
    pub struct Root {
        pub body: Vec<Body>,
        pub meta: MetaEmpty,
    }

    #[derive(Debug, Deserialize)]
    pub struct Body {
        pub dataset: Vec<Dataset>,
        pub key: Timestamp,
        #[serde(rename = "summaryOfValues")]
        pub summary_of_values: f64,
    }

    #[derive(Debug, Deserialize)]
    pub struct Dataset {
        pub key: Timestamp,
        #[serde(with = "default_for_null")]
        pub value: f64,
    }
}

// Deserializer i64 that is a Timestamp or TimestampMilli to a DateTime<Local>
mod timestamp_seconds_milli_or_not {
    use std::{error, fmt};

    use chrono::{DateTime, Local, TimeZone};
    use serde::de::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        // https://www.compuhoy.com/is-unix-timestamp-in-seconds-or-milliseconds/
        match timestamp.checked_ilog10().unwrap_or_default() {
            0..=9 => convert_timestamp(timestamp).map_err(de::Error::custom),
            _ => convert_timestamp(timestamp / 1000).map_err(de::Error::custom),
        }
    }

    pub fn convert_timestamp(timestamp: i64) -> Result<DateTime<Local>, DeTimestampMilliNot> {
        match Local.timestamp_opt(timestamp, 0) {
            chrono::LocalResult::None => Err(DeTimestampMilliNot::Invalid),
            chrono::LocalResult::Single(time) => Ok(time),
            // return latest time
            chrono::LocalResult::Ambiguous(_, time) => Ok(time),
        }
    }

    #[derive(Debug)]
    pub enum DeTimestampMilliNot {
        Invalid,
    }

    impl error::Error for DeTimestampMilliNot {}

    impl fmt::Display for DeTimestampMilliNot {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use DeTimestampMilliNot::*;
            match self {
                Invalid => write!(f, "Invalid timestamp."),
            }
        }
    }
}

mod default_for_null {
    use serde::de::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Default,
    {
        Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};
    use serde::Deserialize;

    use super::{default_for_null, meta::Rel, meta::State, timestamp_seconds_milli_or_not};

    #[derive(Debug, Deserialize)]
    struct TestStructTS {
        #[serde(with = "timestamp_seconds_milli_or_not")]
        pub timestamp: DateTime<Local>,
    }

    #[test]
    fn deserialize_timestamp_milli_or_not() {
        // timestamp in seconds
        let result: Result<TestStructTS, serde_json::Error> =
            serde_json::from_str(r#"{"timestamp": 1678801224}"#);
        assert_eq!(1678801224, result.unwrap().timestamp.timestamp());

        // timestamp in miliseconds
        let result: Result<TestStructTS, serde_json::Error> =
            serde_json::from_str(r#"{"timestamp": 1536127535063}"#);
        assert_eq!(1536127535, result.unwrap().timestamp.timestamp());
    }

    #[derive(Debug, Deserialize)]
    struct TestStructState {
        pub state: State,
    }

    #[test]
    fn deserialize_meta_state() {
        // outdated State
        let result: Result<TestStructState, serde_json::Error> =
            serde_json::from_str(r#"{"state": "OUTDATED"}"#);
        assert_eq!(State::Outdated, result.unwrap().state);

        // synced State
        let result: Result<TestStructState, serde_json::Error> =
            serde_json::from_str(r#"{"state": "SYNCED"}"#);
        assert_eq!(State::Synced, result.unwrap().state);

        // in lover case -> invalid
        let result: Result<TestStructState, serde_json::Error> =
            serde_json::from_str(r#"{"state": "synced"}"#);
        assert!(result.is_err());
    }

    #[derive(Debug, Deserialize)]
    struct TestStructRel {
        pub rel: Rel,
    }

    #[test]
    fn deserialize_meta_rel() {
        // self
        let result: Result<TestStructRel, serde_json::Error> =
            serde_json::from_str(r#"{"rel": "self"}"#);
        assert_eq!(Rel::_Self, result.unwrap().rel);

        // child
        let result: Result<TestStructRel, serde_json::Error> =
            serde_json::from_str(r#"{"rel": "child"}"#);
        assert_eq!(Rel::Child, result.unwrap().rel);

        // invalid
        let result: Result<TestStructRel, serde_json::Error> =
            serde_json::from_str(r#"{"rel": "_Self"}"#);
        assert!(result.is_err());
    }

    #[derive(Debug, Deserialize)]
    struct F64Null {
        #[serde(with = "default_for_null")]
        pub value: f64,
    }

    #[test]
    fn f64_null() {
        let result: Result<F64Null, serde_json::Error> = serde_json::from_str(r#"{"value": null}"#);
        assert_eq!(0.0, result.unwrap().value);

        let result: Result<F64Null, serde_json::Error> =
            serde_json::from_str(r#"{"value": 5000.0}"#);
        assert_eq!(5000.0, result.unwrap().value);
    }
}
