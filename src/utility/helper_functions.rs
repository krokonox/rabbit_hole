use serde::{Deserialize, Deserializer, Serialize, Serializer};
use chrono::{Date, Local, NaiveDate};
use rand::seq::SliceRandom;
use rand::thread_rng; 

pub fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date_str = date.format("%Y-%m-%d").to_string();
    serializer.serialize_str(&date_str)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str: String = Deserialize::deserialize(deserializer)?;
    let date =
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(serde::de::Error::custom)?;
    Ok(date)
}

pub fn get_random_color() -> String {
    let colors = vec!["red", "green", "yellow", "blue", "magenta", "cyan", "white", "purple", "pink", "orange" ];
    let mut rng = rand::thread_rng();
    let color = colors.choose(&mut rng).unwrap();
    color.to_string()
}