// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use crate::{models::move_resources::MoveResource, schema::coin_infos, util::truncate_str};
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
#[diesel(primary_key(coin_type))]
#[diesel(table_name = coin_infos)]
pub struct CoinInfo {
    pub coin_type: String,
    pub transaction_version_created: i64,
    pub creator_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub supply: BigDecimal,
    pub inserted_at: chrono::NaiveDateTime,
}
pub struct CoinInfoType {
    coin_type: String,
    pub creator_address: String,
    name: String,
    symbol: String,
}

impl CoinInfoType {
    pub fn from_move_type(move_type: &MoveType) -> anyhow::Result<Self> {
        let coin_type = move_type.to_string();
        let (address, name, symbol) = if let MoveType::Struct(inner) = move_type {
            (
                inner.address.to_string(),
                inner.module.to_string(),
                inner.name.to_string(),
            )
        } else {
            Err(anyhow::anyhow!("MoveType is not a struct: {:?}", move_type))?
        };
        Ok(Self {
            coin_type,
            creator_address: address,
            name,
            symbol,
        })
    }

    pub fn get_coin_type_trunc(&self) -> String {
        truncate_str(&self.coin_type, 256)
    }
}

impl CoinInfo {
    /// We can find coin info from resources. If the coin info appears multiple times we will only keep the first transaction because it can't be modified.
    pub fn from_write_resource(
        write_resource: &APIWriteResource,
        txn_version: i64,
    ) -> anyhow::Result<Option<Self>> {
        let type_str = format!(
            "{}::{}::{}",
            write_resource.data.typ.address,
            write_resource.data.typ.module,
            write_resource.data.typ.name
        );
        if !CoinResource::is_resource_supported(type_str.as_str()) {
            return Ok(None);
        }
        let resource = MoveResource::from_write_resource(
            write_resource,
            0, // Placeholder, this isn't used anyway
            txn_version,
            0, // Placeholder, this isn't used anyway
        );
        match &CoinResource::from_resource(&type_str, resource.data.as_ref().unwrap(), txn_version)?
        {
            CoinResource::CoinInfoResource(inner) => {
                let coin_info_type =
                    &CoinInfoType::from_move_type(&write_resource.data.typ.generic_type_params[0])?;
                let supply = inner.get_limit().context(format!(
                    "supply limit missing in coin info: {:?}, txn {}",
                    inner, txn_version
                ))?;

                Ok(Some(Self {
                    coin_type: coin_info_type.get_coin_type_trunc(),
                    transaction_version_created: txn_version,
                    creator_address: coin_info_type.creator_address.clone(),
                    name: inner.get_name_trunc(),
                    symbol: inner.get_symbol_trunc(),
                    decimals: inner.decimals,
                    supply,
                    inserted_at: chrono::Utc::now().naive_utc(),
                }))
            }
            _ => Ok(None),
        }
    }
}
