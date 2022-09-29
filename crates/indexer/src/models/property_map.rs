// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_json::Value;
use std::collections::HashMap;

use super::token_models::token_bcs_utils as utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyValue {
    value: String,
    typ: String,
}

pub fn create_property_value(typ: String, value: String) -> Result<PropertyValue> {
    Ok(PropertyValue {
        value: utils::convert_bcs_hex(typ.clone(), value.clone()).unwrap_or(value),
        typ,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyMap {
    data: HashMap<String, PropertyValue>,
}

impl PropertyMap {
    pub fn from_bsc_encode_str(val: Value) -> Option<PropertyMap> {
        let mut pm = PropertyMap {
            data: HashMap::new(),
        };
        let records: &Vec<Value> = val["map"]["data"].as_array()?;
        for entry in records {
            let key = entry["key"].as_str()?;
            let val = entry["value"]["value"].as_str()?;
            let typ = entry["value"]["type"].as_str()?;
            let pv = create_property_value(typ.to_string(), val.to_string()).ok()?;
            pm.data.insert(key.to_string(), pv);
        }
        Some(pm)
    }
}
