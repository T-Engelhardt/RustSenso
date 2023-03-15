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
        #[serde(with = "timestamp_seconds_mill_or_not")]
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
                .and_then(|x| Some(&x.reports))
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
    pub fn find_report<'a>(reports: &'a Vec<Report>, report_id: &'a str) -> Option<&'a Report> {
        reports.iter().find(|r| r.id == report_id)
    }
}

// Deserializer i64 that is a Timestamp or TimestampMilli to a DateTime<Local>
mod timestamp_seconds_mill_or_not {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        // https://www.compuhoy.com/is-unix-timestamp-in-seconds-or-milliseconds/
        match timestamp.checked_ilog10().unwrap_or_default() {
            0..=9 => Ok(Local.timestamp_opt(timestamp, 0).unwrap()),
            _ => Ok(Local.timestamp_opt(timestamp / 1000, 0).unwrap()),
        }
    }
}
