// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This is required because a diesel macro makes clippy sad
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::unused_unit)]

use std::collections::HashMap;

use crate::{
    schema::current_ans_lookup,
    util::{bigdecimal_to_u64, parse_timestamp_secs},
};
use aptos_api_types::{deserialize_from_string, Transaction as APITransaction};
use bigdecimal::BigDecimal;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

pub const ADDR: &str = "0xdbf606fea404cb26efe68d00f8f4fff8e4b9ce69f903818f8acf81473a32430a";
type Domain = String;
type Subdomain = String;
// PK of current_ans_lookup, i.e. domain and subdomain name
pub type CurrentAnsLookupPK = (Domain, Subdomain);

#[derive(Debug, Deserialize, FieldCount, Identifiable, Insertable, Queryable, Serialize)]
#[diesel(primary_key(domain, subdomain))]
#[diesel(table_name = current_ans_lookup)]
pub struct CurrentAnsLookup {
    pub domain: String,
    pub subdomain: String,
    pub registered_address: Option<String>,
    pub last_transaction_version: i64,
    pub expiration_timestamp: chrono::NaiveDateTime,
    pub inserted_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNameAddressEventV1 {
    subdomain_name: OptionalString,
    domain_name: String,
    new_address: OptionalString,
    #[serde(deserialize_with = "deserialize_from_string")]
    expiration_time_secs: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RegisterNameEventV1 {
    subdomain_name: OptionalString,
    domain_name: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    expiration_time_secs: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OptionalString {
    vec: Vec<String>,
}

impl OptionalString {
    fn get_string(&self) -> Option<String> {
        if self.vec.is_empty() {
            None
        } else {
            Some(self.vec[0].clone())
        }
    }
}

impl CurrentAnsLookup {
    pub fn from_transaction(transaction: &APITransaction) -> HashMap<CurrentAnsLookupPK, Self> {
        let mut current_ans_lookups: HashMap<CurrentAnsLookupPK, Self> = HashMap::new();
        if let APITransaction::UserTransaction(user_txn) = transaction {
            for event in &user_txn.events {
                let txn_version = user_txn.info.version.0 as i64;
                let event_type = event.typ.to_string();
                let current_ans_lookup: Self;
                // TODO: Replace this section with a match
                if event_type.as_str() == format!("{}::events::SetNameAddressEventV1", ADDR) {
                    let parsed_event: SetNameAddressEventV1 =
                        serde_json::from_value(event.data.clone()).expect(
                            format!(
                                "Failed to deserialize SetNameAddressEventV1, {:?}",
                                &event.data
                            )
                            .as_str(),
                        );
                    let expiration_timestamp = parse_timestamp_secs(
                        bigdecimal_to_u64(&parsed_event.expiration_time_secs),
                        txn_version,
                    );
                    current_ans_lookup = Self {
                        domain: parsed_event.domain_name,
                        subdomain: parsed_event.subdomain_name.get_string().unwrap_or_default(),
                        registered_address: parsed_event.new_address.get_string(),
                        last_transaction_version: txn_version,
                        expiration_timestamp,
                        inserted_at: chrono::Utc::now().naive_utc(),
                    };
                    current_ans_lookups.insert(
                        (
                            current_ans_lookup.domain.clone(),
                            current_ans_lookup.subdomain.clone(),
                        ),
                        current_ans_lookup,
                    );
                } else if event_type.as_str() == format!("{}::events::RegisterNameEventV1", ADDR) {
                    let parsed_event: RegisterNameEventV1 =
                        serde_json::from_value(event.data.clone()).expect(
                            format!(
                                "Failed to deserialize SetNameAddressEventV1, {:?}",
                                &event.data
                            )
                            .as_str(),
                        );
                    let expiration_timestamp = parse_timestamp_secs(
                        bigdecimal_to_u64(&parsed_event.expiration_time_secs),
                        txn_version,
                    );
                    current_ans_lookup = Self {
                        domain: parsed_event.domain_name,
                        subdomain: parsed_event.subdomain_name.get_string().unwrap_or_default(),
                        registered_address: None,
                        last_transaction_version: txn_version,
                        expiration_timestamp,
                        inserted_at: chrono::Utc::now().naive_utc(),
                    };
                    current_ans_lookups.insert(
                        (
                            current_ans_lookup.domain.clone(),
                            current_ans_lookup.subdomain.clone(),
                        ),
                        current_ans_lookup,
                    );
                }
            }
        }
        current_ans_lookups
    }
}
