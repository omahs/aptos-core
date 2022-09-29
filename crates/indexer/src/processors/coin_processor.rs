// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    database::{
        clean_data_for_db, execute_with_better_error, get_chunks, PgDbPool, PgPoolConnection,
    },
    indexer::{
        errors::TransactionProcessingError, processing_result::ProcessingResult,
        transaction_processor::TransactionProcessor,
    },
    models::coin_models::coin_infos::CoinInfo,
    schema,
};
use aptos_api_types::{
    DeleteTableItem as APIDeleteTableItem, Transaction as APITransaction,
    WriteResource as APIWriteResource, WriteSetChange as APIWriteSetChange,
    WriteTableItem as APIWriteTableItem,
};
use async_trait::async_trait;
use diesel::{pg::upsert::excluded, result::Error, ExpressionMethods, PgConnection};
use field_count::FieldCount;
use std::{collections::HashMap, fmt::Debug};

pub const NAME: &str = "coin_processor";
pub struct CoinTransactionProcessor {
    connection_pool: PgDbPool,
}

impl CoinTransactionProcessor {
    pub fn new(connection_pool: PgDbPool) -> Self {
        Self { connection_pool }
    }
}

impl Debug for CoinTransactionProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "CoinTransactionProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

fn insert_to_db_impl(conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
    Ok(())
}

fn insert_to_db(
    conn: &mut PgPoolConnection,
    name: &'static str,
    start_version: u64,
    end_version: u64,
) -> Result<(), diesel::result::Error> {
    aptos_logger::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );
    match conn
        .build_transaction()
        .read_write()
        .run::<_, Error, _>(|pg_conn| insert_to_db_impl(pg_conn))
    {
        Ok(_) => Ok(()),
        Err(_) => conn
            .build_transaction()
            .read_write()
            .run::<_, Error, _>(|pg_conn| {
                // let tokens = clean_data_for_db(tokens, true);

                insert_to_db_impl(pg_conn)
            }),
    }
}

#[async_trait]
impl TransactionProcessor for CoinTransactionProcessor {
    fn name(&self) -> &'static str {
        NAME
    }

    async fn process_transactions(
        &self,
        transactions: Vec<APITransaction>,
        start_version: u64,
        end_version: u64,
    ) -> Result<ProcessingResult, TransactionProcessingError> {
        for txn in transactions {
            if let APITransaction::GenesisTransaction(genesis_txn) = &txn {
                for wsc in &genesis_txn.info.changes {
                    if let APIWriteSetChange::WriteResource(write_resource) = wsc {
                        let coin_info = CoinInfo::from_write_resource(
                            &write_resource,
                            genesis_txn.info.version.0 as i64,
                        )
                        .unwrap();
                    }
                }
            }
            if let APITransaction::UserTransaction(user_txn) = &txn {
                for wsc in &user_txn.info.changes {
                    if let APIWriteSetChange::WriteResource(write_resource) = wsc {
                        let coin_info = CoinInfo::from_write_resource(
                            &write_resource,
                            user_txn.info.version.0 as i64,
                        )
                        .unwrap();
                    }
                }
            }
        }
        let mut conn = self.get_conn();
        let tx_result = insert_to_db(&mut conn, self.name(), start_version, end_version);
        match tx_result {
            Ok(_) => Ok(ProcessingResult::new(
                self.name(),
                start_version,
                end_version,
            )),
            Err(err) => Err(TransactionProcessingError::TransactionCommitError((
                anyhow::Error::from(err),
                start_version,
                end_version,
                self.name(),
            ))),
        }
    }

    fn connection_pool(&self) -> &PgDbPool {
        &self.connection_pool
    }
}
