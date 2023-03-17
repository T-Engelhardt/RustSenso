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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};
    use serde::Deserialize;

    use super::{meta::Rel, meta::State, timestamp_seconds_milli_or_not};

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
}
