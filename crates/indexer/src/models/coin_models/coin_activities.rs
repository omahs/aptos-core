// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use crate::{models::move_resources::MoveResource, schema::coin_activities, util::truncate_str};
use anyhow::Context;
use aptos_api_types::{
    DeleteTableItem as APIDeleteTableItem, MoveType, Transaction as APITransaction,
    WriteResource as APIWriteResource, WriteSetChange as APIWriteSetChange,
    WriteTableItem as APIWriteTableItem,
};
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

use super::coin_utils::CoinResource;

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Queryable, Serialize)]
#[diesel(primary_key(transaction_version, event_account_address, event_creation_number, event_sequence_number))]
#[diesel(table_name = coin_activities)]
pub struct CoinActivity {
    pub transaction_version: i64,
    pub event_account_address: String,
    pub event_creation_number: i64,
    pub event_sequence_number: i64,
    pub owner_address: String,
    pub coin_type: String,
    pub amount: BigDecimal,
    pub activity_type: String,
    pub is_gas_fee: bool,
    pub is_transaction_success: bool,
    pub entry_function_id_str: String,
    pub inserted_at: chrono::NaiveDateTime,
}

/// Coin information is mostly in Resources but some pieces are in table items (e.g. supply from aggregator table)
pub struct CoinSupply {
    pub coin_type: String,
    pub transaction_version_created: i64,
    pub creator_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub supply: BigDecimal,
}

impl CoinActivity {
    /// There are different objects containing different information about balances and coins. 
    /// Events: Withdraw and Deposit event containing amounts. There is no coin type so we need to get that from Resources. (from event guid)
    /// CoinInfo Resource: Contains name, symbol, decimals and supply. (if supply is aggregator, however, actual supply amount will live in a separate table)
    /// CoinStore Resource: Contains
    pub fn from_transactions(
        txn: &APITransaction,
        txn_version: i64,
    ) -> anyhow::Result<Self> {
        get_write_set
        
        Ok(None)
    }
}
