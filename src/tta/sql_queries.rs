use std::collections::{self};

use chrono::Utc;
use sqlx::{Pool, Postgres};
use tracing::{info, instrument};

use crate::tta::Transaction;

#[derive(Debug, Clone)]
pub struct SqlClient {
    pool: Pool<Postgres>,
}

impl SqlClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn get_outgoing_txns(
        &self,
        accounts: collections::HashSet<String>,
        start_date: chrono::DateTime<Utc>,
        end_date: chrono::DateTime<Utc>,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let accs: Vec<String> = accounts.into_iter().collect();
        let s = start_date.to_rfc3339();
        let e = end_date.to_rfc3339();

        let txs: Vec<Transaction> = sqlx::query_as!(
            Transaction,
            r##"SELECT
                T.TRANSACTION_HASH as T_TRANSACTION_HASH,
                T.INCLUDED_IN_BLOCK_HASH as T_INCLUDED_IN_BLOCK_HASH,
                T.INCLUDED_IN_CHUNK_HASH as T_INCLUDED_IN_CHUNK_HASH,
                T.INDEX_IN_CHUNK as T_INDEX_IN_CHUNK,
                T.BLOCK_TIMESTAMP as T_BLOCK_TIMESTAMP,
                T.SIGNER_ACCOUNT_ID as T_SIGNER_ACCOUNT_ID,
                T.SIGNER_PUBLIC_KEY as T_SIGNER_PUBLIC_KEY,
                T.NONCE as T_NONCE,
                T.RECEIVER_ACCOUNT_ID as T_RECEIVER_ACCOUNT_ID,
                T.SIGNATURE as T_SIGNATURE,
                T.STATUS as "t_status: String",
                T.CONVERTED_INTO_RECEIPT_ID as T_CONVERTED_INTO_RECEIPT_ID,
                T.RECEIPT_CONVERSION_GAS_BURNT as T_RECEIPT_CONVERSION_GAS_BURNT,
                T.RECEIPT_CONVERSION_TOKENS_BURNT as T_RECEIPT_CONVERSION_TOKENS_BURNT,
                R.RECEIPT_ID as R_RECEIPT_ID,
                R.INCLUDED_IN_BLOCK_HASH as R_INCLUDED_IN_BLOCK_HASH,
                R.INCLUDED_IN_CHUNK_HASH as R_INCLUDED_IN_CHUNK_HASH,
                R.INDEX_IN_CHUNK as R_INDEX_IN_CHUNK,
                R.INCLUDED_IN_BLOCK_TIMESTAMP as R_INCLUDED_IN_BLOCK_TIMESTAMP,
                R.PREDECESSOR_ACCOUNT_ID as R_PREDECESSOR_ACCOUNT_ID,
                R.RECEIVER_ACCOUNT_ID as R_RECEIVER_ACCOUNT_ID,
                R.RECEIPT_KIND as "r_receipt_kind: String",
                R.ORIGINATED_FROM_TRANSACTION_HASH as R_ORIGINATED_FROM_TRANSACTION_HASH,
                TA.TRANSACTION_HASH as TA_TRANSACTION_HASH,
                TA.INDEX_IN_TRANSACTION as TA_INDEX_IN_TRANSACTION,
                TA.ACTION_KIND as "ta_action_kind: String",
                TA.ARGS as TA_ARGS,
                ARA.RECEIPT_ID as ARA_RECEIPT_ID,
                ARA.INDEX_IN_ACTION_RECEIPT as ARA_INDEX_IN_ACTION_RECEIPT,
                ARA.ARGS as ARA_ARGS,
                ARA.RECEIPT_PREDECESSOR_ACCOUNT_ID as ARA_RECEIPT_PREDECESSOR_ACCOUNT_ID,
                ARA.RECEIPT_RECEIVER_ACCOUNT_ID as ARA_RECEIPT_RECEIVER_ACCOUNT_ID,
                ARA.RECEIPT_INCLUDED_IN_BLOCK_TIMESTAMP as ARA_RECEIPT_INCLUDED_IN_BLOCK_TIMESTAMP,
                ARA.ACTION_KIND as "ara_action_kind: String",
                B.BLOCK_HEIGHT as B_BLOCK_HEIGHT,
                B.BLOCK_HASH as B_BLOCK_HASH,
                B.PREV_BLOCK_HASH as B_PREV_BLOCK_HASH,
                B.BLOCK_TIMESTAMP as B_BLOCK_TIMESTAMP,
                B.GAS_PRICE as B_GAS_PRICE,
                B.AUTHOR_ACCOUNT_ID as B_AUTHOR_ACCOUNT_ID,
                EO.RECEIPT_ID as EO_RECEIPT_ID,
                EO.EXECUTED_IN_BLOCK_HASH  as EO_EXECUTED_IN_BLOCK_HASH ,
                EO.EXECUTED_IN_BLOCK_TIMESTAMP as EO_EXECUTED_IN_BLOCK_TIMESTAMP,
                EO.INDEX_IN_CHUNK as EO_INDEX_IN_CHUNK,
                EO.GAS_BURNT as EO_GAS_BURNT,
                EO.TOKENS_BURNT as EO_TOKENS_BURNT,
                EO.EXECUTOR_ACCOUNT_ID as EO_EXECUTOR_ACCOUNT_ID,
                EO.SHARD_ID as EO_SHARD_ID,
                EO.STATUS as "eo_status: String"
            FROM
                TRANSACTIONS T
                LEFT JOIN RECEIPTS R ON (T.CONVERTED_INTO_RECEIPT_ID = R.RECEIPT_ID
                        OR t.TRANSACTION_HASH = R.ORIGINATED_FROM_TRANSACTION_HASH)
                LEFT JOIN TRANSACTION_ACTIONS TA ON T.TRANSACTION_HASH = TA.TRANSACTION_HASH
                LEFT JOIN ACTION_RECEIPT_ACTIONS ARA ON ARA.RECEIPT_ID = R.RECEIPT_ID
                LEFT JOIN BLOCKS B ON B.BLOCK_HASH = R.INCLUDED_IN_BLOCK_HASH
                LEFT JOIN EXECUTION_OUTCOMES EO ON EO.RECEIPT_ID = R.RECEIPT_ID
            WHERE
                receipt_predecessor_account_id = ANY($1)
                AND EO.STATUS IN ('SUCCESS_RECEIPT_ID', 'SUCCESS_VALUE')
                and to_char(to_timestamp(b.block_timestamp / 1000000000), 'YYYY-MM-DD"T"HH24:MI:SS"Z"') >= $2
                and to_char(to_timestamp(b.block_timestamp / 1000000000), 'YYYY-MM-DD"T"HH24:MI:SS"Z"') < $3;
            "##,
            &accs,
            &s,
            &e,
        )
        .fetch_all(&self.pool)
        .await?;

        info!("Got {} transactions", txs.len());

        Ok(txs)
    }

    #[instrument(skip(self))]
    pub async fn get_incoming_txns(
        &self,
        accounts: collections::HashSet<String>,
        start_date: chrono::DateTime<Utc>,
        end_date: chrono::DateTime<Utc>,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let accs: Vec<String> = accounts.into_iter().collect();
        let s = start_date.to_rfc3339();
        let e = end_date.to_rfc3339();

        let txs: Vec<Transaction> = sqlx::query_as!(
            Transaction,
            r##"
            SELECT
                T.TRANSACTION_HASH as T_TRANSACTION_HASH,
                T.INCLUDED_IN_BLOCK_HASH as T_INCLUDED_IN_BLOCK_HASH,
                T.INCLUDED_IN_CHUNK_HASH as T_INCLUDED_IN_CHUNK_HASH,
                T.INDEX_IN_CHUNK as T_INDEX_IN_CHUNK,
                T.BLOCK_TIMESTAMP as T_BLOCK_TIMESTAMP,
                T.SIGNER_ACCOUNT_ID as T_SIGNER_ACCOUNT_ID,
                T.SIGNER_PUBLIC_KEY as T_SIGNER_PUBLIC_KEY,
                T.NONCE as T_NONCE,
                T.RECEIVER_ACCOUNT_ID as T_RECEIVER_ACCOUNT_ID,
                T.SIGNATURE as T_SIGNATURE,
                T.STATUS as "t_status: String",
                T.CONVERTED_INTO_RECEIPT_ID as T_CONVERTED_INTO_RECEIPT_ID,
                T.RECEIPT_CONVERSION_GAS_BURNT as T_RECEIPT_CONVERSION_GAS_BURNT,
                T.RECEIPT_CONVERSION_TOKENS_BURNT as T_RECEIPT_CONVERSION_TOKENS_BURNT,
                R.RECEIPT_ID as R_RECEIPT_ID,
                R.INCLUDED_IN_BLOCK_HASH as R_INCLUDED_IN_BLOCK_HASH,
                R.INCLUDED_IN_CHUNK_HASH as R_INCLUDED_IN_CHUNK_HASH,
                R.INDEX_IN_CHUNK as R_INDEX_IN_CHUNK,
                R.INCLUDED_IN_BLOCK_TIMESTAMP as R_INCLUDED_IN_BLOCK_TIMESTAMP,
                R.PREDECESSOR_ACCOUNT_ID as R_PREDECESSOR_ACCOUNT_ID,
                R.RECEIVER_ACCOUNT_ID as R_RECEIVER_ACCOUNT_ID,
                R.RECEIPT_KIND as "r_receipt_kind: String",
                R.ORIGINATED_FROM_TRANSACTION_HASH as R_ORIGINATED_FROM_TRANSACTION_HASH,
                TA.TRANSACTION_HASH as TA_TRANSACTION_HASH,
                TA.INDEX_IN_TRANSACTION as TA_INDEX_IN_TRANSACTION,
                TA.ACTION_KIND as "ta_action_kind: String",
                TA.ARGS as TA_ARGS,
                ARA.RECEIPT_ID as ARA_RECEIPT_ID,
                ARA.INDEX_IN_ACTION_RECEIPT as ARA_INDEX_IN_ACTION_RECEIPT,
                ARA.ARGS as ARA_ARGS,
                ARA.RECEIPT_PREDECESSOR_ACCOUNT_ID as ARA_RECEIPT_PREDECESSOR_ACCOUNT_ID,
                ARA.RECEIPT_RECEIVER_ACCOUNT_ID as ARA_RECEIPT_RECEIVER_ACCOUNT_ID,
                ARA.RECEIPT_INCLUDED_IN_BLOCK_TIMESTAMP as ARA_RECEIPT_INCLUDED_IN_BLOCK_TIMESTAMP,
                ARA.ACTION_KIND as "ara_action_kind: String",
                B.BLOCK_HEIGHT as B_BLOCK_HEIGHT,
                B.BLOCK_HASH as B_BLOCK_HASH,
                B.PREV_BLOCK_HASH as B_PREV_BLOCK_HASH,
                B.BLOCK_TIMESTAMP as B_BLOCK_TIMESTAMP,
                B.GAS_PRICE as B_GAS_PRICE,
                B.AUTHOR_ACCOUNT_ID as B_AUTHOR_ACCOUNT_ID,
                EO.RECEIPT_ID as EO_RECEIPT_ID,
                EO.EXECUTED_IN_BLOCK_HASH  as EO_EXECUTED_IN_BLOCK_HASH ,
                EO.EXECUTED_IN_BLOCK_TIMESTAMP as EO_EXECUTED_IN_BLOCK_TIMESTAMP,
                EO.INDEX_IN_CHUNK as EO_INDEX_IN_CHUNK,
                EO.GAS_BURNT as EO_GAS_BURNT,
                EO.TOKENS_BURNT as EO_TOKENS_BURNT,
                EO.EXECUTOR_ACCOUNT_ID as EO_EXECUTOR_ACCOUNT_ID,
                EO.SHARD_ID as EO_SHARD_ID,
                EO.STATUS as "eo_status: String"
            FROM
                TRANSACTIONS T
                LEFT JOIN RECEIPTS R ON (T.CONVERTED_INTO_RECEIPT_ID = R.RECEIPT_ID
                        OR T.TRANSACTION_HASH = R.ORIGINATED_FROM_TRANSACTION_HASH)
                LEFT JOIN TRANSACTION_ACTIONS TA ON T.TRANSACTION_HASH = TA.TRANSACTION_HASH
                LEFT JOIN ACTION_RECEIPT_ACTIONS ARA ON ARA.RECEIPT_ID = R.RECEIPT_ID
                LEFT JOIN BLOCKS B ON B.BLOCK_HASH = R.INCLUDED_IN_BLOCK_HASH
                LEFT JOIN EXECUTION_OUTCOMES EO ON EO.RECEIPT_ID = R.RECEIPT_ID
            WHERE
                RECEIPT_RECEIVER_ACCOUNT_ID = ANY ($1)
                AND EO.STATUS IN ('SUCCESS_RECEIPT_ID', 'SUCCESS_VALUE')
                AND TO_CHAR(TO_TIMESTAMP(B.BLOCK_TIMESTAMP / 1000000000), 'YYYY-MM-DD"T"HH24:MI:SS"Z"') >= $2
                AND TO_CHAR(TO_TIMESTAMP(B.BLOCK_TIMESTAMP / 1000000000), 'YYYY-MM-DD"T"HH24:MI:SS"Z"') < $3;
            "##,
            &accs,
            &s,
            &e,
        )
        .fetch_all(&self.pool)
        .await?;

        info!("Got {} transactions", txs.len());
        Ok(txs)
    }
}
