use std::collections::HashMap;
use rocket_dyn_templates::tera::{Result, Value};
use crate::db::users;

pub fn split_by_index(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let input = value.as_str().unwrap_or_default();
    let index= args.get("index").unwrap().as_i64().unwrap() as usize;
    Ok(Value::String(input.split_at(index).0.to_string()))
}

pub fn user_username(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let value= value.to_string();
    let user_id = value.as_str();
    let user_data = users::get_user("id", user_id);
    match user_data {
        Ok(Some(user_data)) => {
            Ok(Value::String(user_data.username))
        },
        Ok(None) => Ok(Value::String("None".to_string())),
        Err(err) => {
            Ok(Value::String(err.to_string()))
        }
    }
}

pub fn date_ago(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let days = value.as_str().unwrap_or_default();
    // Date is in UTC format 2023-03-17T11:44:24
    // We want to get the number of days ago
    let parsed_date = chrono::NaiveDateTime::parse_from_str(days, "%Y-%m-%dT%H:%M:%S").unwrap();
    let now = chrono::Utc::now().naive_utc();
    let days = now.signed_duration_since(parsed_date).num_days();
    if days == 0 {
        return Ok(Value::String("today".to_string()))
    } else if days == 1 {
        return Ok(Value::String("yesterday".to_string()))
    }
    Ok(Value::String(format!("{} days ago", days)))
}