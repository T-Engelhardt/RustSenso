#![allow(dead_code)]
use serde::Deserialize;

// https://transform.tools/json-to-rust-serde
// Property Name Format snake_case

// META
pub mod meta {

    use super::*;
    use chrono::{DateTime, Local};
    use serde_with::serde_as;
    use serde_with::TimestampMilliSeconds;

    #[derive(Debug, Deserialize)]
    pub struct MetaEmpty {}

    #[derive(Debug, Deserialize)]
    pub struct Meta {
        #[serde(rename = "resourceState")]
        pub resource_state: Vec<ResourceState>,
    }

    #[serde_as]
    #[derive(Debug, Deserialize)]
    pub struct ResourceState {
        pub link: Link,
        pub state: State,
        #[serde_as(as = "TimestampMilliSeconds<i64>")]
        pub timestamp: DateTime<Local>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Link {
        pub rel: Rel,
        #[serde(rename = "resourceLink")]
        pub resource_link: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub enum State {
        OUTDATED,
        SYNCED,
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
