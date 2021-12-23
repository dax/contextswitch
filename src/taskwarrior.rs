use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json;
use std::io::Error;
use std::process::Command;
use std::str;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Recurrence {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Completed,
    Recurring,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ContextSwitchMetadata {
    pub test: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub uuid: Uuid,
    pub id: u32,
    #[serde(with = "tw_date_format")]
    pub entry: DateTime<Utc>,
    #[serde(with = "tw_date_format")]
    pub modified: DateTime<Utc>,
    pub status: Status,
    pub description: String,
    pub urgency: f64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_tw_date_format"
    )]
    pub due: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_tw_date_format"
    )]
    pub end: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recur: Option<Recurrence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_from_json"
    )]
    pub contextswitch: Option<ContextSwitchMetadata>,
}

fn deserialize_from_json<'de, D>(deserializer: D) -> Result<Option<ContextSwitchMetadata>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(de::Error::custom)
}

pub mod tw_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y%m%dT%H%M%SZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

pub mod opt_tw_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y%m%dT%H%M%SZ";

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *date {
            return serializer.serialize_str(&d.format(FORMAT).to_string());
        }

        serializer.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

pub fn export(filters: Vec<&str>) -> Result<Vec<Task>, Error> {
    let mut args = vec!["export"];
    args.extend(filters);
    let export_output = Command::new("task").args(args).output()?;

    let tasks: Vec<Task> = serde_json::from_slice(&export_output.stdout)?;

    return Ok(tasks);
}

pub fn add(add_args: Vec<&str>) -> Result<(), Error> {
    let mut args = vec!["add"];
    args.extend(add_args);
    Command::new("task").args(args).output()?;
    return Ok(());
}
