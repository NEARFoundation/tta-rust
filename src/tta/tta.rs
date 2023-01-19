use std::{
    collections::{self, HashSet},
    vec,
};

use chrono::DateTime;
use tracing::{info, instrument};

use super::{sql_queries::SqlClient, Transaction, TtaError};

#[derive(Debug, Clone)]
pub struct TTA {
    sql_client: SqlClient,
}

impl TTA {
    pub fn new(sql_client: SqlClient) -> Self {
        Self { sql_client }
    }

    #[instrument(skip(self, start_date, end_date, accounts))]
    pub(crate) async fn get_txns_report(
        &self,
        start_date: DateTime<chrono::Utc>,
        end_date: DateTime<chrono::Utc>,
        accounts: HashSet<String>,
    ) -> Result<(), TtaError> {
        info!(?start_date, ?end_date, ?accounts, "Got request");

        let mut join_handles = vec![];

        for acc in accounts {
            let t = self.clone();
            let mut wallets_for_account = collections::HashSet::new();
            wallets_for_account.insert(acc.clone());
            // TODO(pierre): add lockup account

            let w2 = wallets_for_account.clone();
            let t2 = t.clone();
            let task_1 = tokio::spawn(async move {
                match t
                    .handle_incoming_txns(wallets_for_account, start_date, end_date)
                    .await
                {
                    Ok(txns) => Ok(txns),
                    Err(e) => Err(e),
                }
            });
            let task_2 = tokio::spawn(async move {
                match t2.handle_outgoing_txns(w2, start_date, end_date).await {
                    Ok(txns) => Ok(txns),
                    Err(e) => Err(e),
                }
            });
            join_handles.push(task_1);
            join_handles.push(task_2);
        }
        // Wait for threads to be over.
        for ele in join_handles {
            match ele.await {
                Ok(res) => match res {
                    Ok(_) => {}
                    Err(e) => {
                        info!(?e, "Got error");
                    }
                },
                Err(e) => {
                    info!(?e, "Got error");
                }
            }
        }

        info!("Done");

        Ok(())
    }

    async fn handle_incoming_txns(
        self,
        accounts: HashSet<String>,
        start_date: DateTime<chrono::Utc>,
        end_date: DateTime<chrono::Utc>,
    ) -> Result<Vec<Transaction>, TtaError> {
        match self
            .sql_client
            .get_incoming_txns(accounts, start_date, end_date)
            .await
        {
            Ok(txns) => Ok(txns),
            Err(e) => {
                info!(?e, "Got error");
                Err(TtaError::DatabaseError(e))
            }
        }
    }

    async fn handle_outgoing_txns(
        self,
        accounts: HashSet<String>,
        start_date: DateTime<chrono::Utc>,
        end_date: DateTime<chrono::Utc>,
    ) -> Result<Vec<Transaction>, TtaError> {
        match self
            .sql_client
            .get_outgoing_txns(accounts, start_date, end_date)
            .await
        {
            Ok(txns) => Ok(txns),
            Err(e) => Err(TtaError::DatabaseError(e)),
        }
    }
}