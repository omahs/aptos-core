// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]

use crate::util::{hash_str, truncate_str};
use anyhow::{Context, Result};
use aptos_api_types::deserialize_from_string;
use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

/**
 * This file defines deserialized coin types as defined in our 0x1 contracts.
 */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoinInfoResource {
    name: String,
    symbol: String,
    pub decimals: i32,
    pub supply: OptionalAggregatorWrapperResource,
}

impl CoinInfoResource {
    pub fn get_name_trunc(&self) -> String {
        truncate_str(&self.name, 32)
    }

    pub fn get_symbol_trunc(&self) -> String {
        truncate_str(&self.symbol, 10)
    }

    pub fn get_limit(&self) -> Option<BigDecimal> {
        let maybe_supply = self.supply.vec.get(0);
        if let Some(supply) = maybe_supply {
            supply.get_limit()
        } else {
            None
        }
    }

    pub fn get_value(&self) -> Option<BigDecimal> {
        let maybe_supply = self.supply.vec.get(0);
        if let Some(supply) = maybe_supply {
            supply.get_value()
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionalAggregatorWrapperResource {
    pub vec: Vec<OptionalAggregatorResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionalAggregatorResource {
    pub aggregator: AggregatorWrapperResource,
    pub integer: IntegerWrapperResource,
}

impl OptionalAggregatorResource {
    pub fn get_limit(&self) -> Option<BigDecimal> {
        let maybe_limit = self.integer.get_limit();
        if maybe_limit.is_none() {
            self.aggregator.get_limit()
        } else {
            maybe_limit
        }
    }

    pub fn get_value(&self) -> Option<BigDecimal> {
        self.integer.get_value()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatorWrapperResource {
    pub vec: Vec<AggregatorResource>,
}

impl AggregatorWrapperResource {
    pub fn get_limit(&self) -> Option<BigDecimal> {
        self.vec.get(0).map(|inner| inner.limit.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntegerWrapperResource {
    pub vec: Vec<IntegerResource>,
}

impl IntegerWrapperResource {
    pub fn get_value(&self) -> Option<BigDecimal> {
        self.vec.get(0).map(|inner| inner.value.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatorResource {
    pub handle: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntegerResource {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub value: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CoinResource {
    CoinInfoResource(CoinInfoResource),
}

impl CoinResource {
    pub fn is_resource_supported(data_type: &str) -> bool {
        matches!(data_type, "0x1::coin::CoinInfo")
    }

    pub fn from_resource(
        data_type: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<CoinResource> {
        match data_type {
            "0x1::coin::CoinInfo" => serde_json::from_value(data.clone())
                .map(|inner| Some(CoinResource::CoinInfoResource(inner))),
            _ => Ok(None),
        }
        .context(format!(
            "version {} failed! failed to parse type {}, data {:?}",
            txn_version, data_type, data
        ))?
        .context(format!(
            "Resource unsupported! Call is_resource_supported first. version {} type {}",
            txn_version, data_type
        ))
    }
}

