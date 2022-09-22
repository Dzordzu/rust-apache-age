use crate::AgType;
use async_trait_with_sync::async_trait;
use serde::Serialize;
use tokio_postgres::{
    connect,
    tls::{MakeTlsConnect, TlsConnect},
    Socket,
};

use super::constants::*;

use std::sync::Arc;
use tokio::sync::Mutex;
pub use tokio::task::JoinHandle;
pub use tokio_postgres::{Client, Error, Statement};

#[async_trait(Sync)]
/// Handles connecting, configuring and querying graph dbs within postgres instance
pub trait AgeClient: Sync {
    async fn connect_age<T>(
        params: &str,
        tls_mode: T,
    ) -> Arc<Mutex<Result<(Client, JoinHandle<()>), tokio_postgres::Error>>>
    where
        T: MakeTlsConnect<Socket> + 'static + Send + Sync,
        T::TlsConnect: Send + Sync,
        T::Stream: Send + Sync,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send + Sync;

    /// Create a new constraint for the certain label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    async fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str,
    ) -> Arc<Result<u64, postgres::Error>>;

    /// Create unique index for the certain field for the label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    async fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Arc<Result<u64, postgres::Error>>;

    async fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Arc<Result<u64, postgres::Error>>;

    async fn create_graph(&mut self, name: &str) -> Arc<Result<u64, postgres::Error>>;
    async fn drop_graph(&mut self, name: &str) -> Arc<Result<u64, postgres::Error>>;
    async fn graph_exists(&mut self, name: &str) -> Arc<Result<bool, postgres::Error>>;

    /// Exexute cypher query, without any rows to be retured
    async fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Arc<Result<u64, postgres::Error>>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send;

    /// Query cypher for a single agtype (in a format of json)
    ///
    /// **IMPORTANT**: You need to return result of the query as a map
    ///
    /// Example:
    /// ```cypher
    /// MATCH (n: Person) WHERE n.name = 'Alfred' RETURN {name: n.name, surname: n.surname}
    /// ```
    async fn query_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Arc<Result<Vec<postgres::Row>, postgres::Error>>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send;

    /// Prepare cypher query for future use
    async fn prepare_cypher(
        &mut self,
        graph: &str,
        cypher: &str,
        use_arg: bool,
    ) -> Arc<Result<Statement, postgres::Error>>;
}

#[async_trait(Sync)]
impl AgeClient for Client {
    async fn create_graph(&mut self, name: &str) -> Arc<Result<u64, postgres::Error>> {
        Arc::new(self.execute(CREATE_GRAPH, &[&name]).await)
    }

    async fn drop_graph(&mut self, name: &str) -> Arc<Result<u64, postgres::Error>> {
        Arc::new(self.execute(DROP_GRAPH, &[&name]).await)
    }

    async fn connect_age<T>(
        params: &str,
        tls_mode: T,
    ) -> Arc<Mutex<Result<(Client, JoinHandle<()>), tokio_postgres::Error>>>
    where
        T: MakeTlsConnect<Socket> + 'static + Send + Sync,
        T::TlsConnect: Send + Sync,
        T::Stream: Send + Sync,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send + Sync,
    {
        let new_connection = match connect(params, tls_mode).await {
            Ok(x) => x,
            Err(e) => return Arc::new(Mutex::new(Err(e))),
        };

        let (client, connection) = new_connection;

        let handle = tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        for query in [
            client.simple_query(LOAD_AGE).await,
            client.simple_query(SET_AGE).await,
        ] {
            if let Err(err) = query {
                return Arc::new(Mutex::new(Err(err)));
            };
        }
        Arc::new(Mutex::new(Ok((client, handle))))
    }

    async fn query_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Arc<Result<Vec<postgres::Row>, postgres::Error>>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send,
    {
        let query_result = match agtype {
            Some(x) => {
                let query = format!(cypher_query!(), graph, cypher, CQ_ARG);
                self.query(&query, &[&x]).await
            }
            None => {
                let query = format!(cypher_query!(), graph, cypher, CQ_NO_ARG);
                self.query(&query, &[]).await
            }
        };

        Arc::new(query_result)
    }

    async fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str,
    ) -> Arc<Result<u64, postgres::Error>> {
        let query = format!(constraint!(), graph, label, name, constraint_text);

        Arc::new(self.execute(&query, &[]).await)
    }

    async fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Arc<Result<u64, postgres::Error>> {
        let query = format!(unique_index!(), name, graph, label, field);

        Arc::new(self.execute(&query, &[]).await)
    }

    async fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Arc<Result<u64, postgres::Error>> {
        self.constraint(graph, label, name, &format!(required_constraint!(), field))
            .await
    }

    async fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Arc<Result<u64, postgres::Error>>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send,
    {
        Arc::new(match agtype {
            Some(x) => {
                let query = format!(cypher_query!(), graph, cypher, CQ_ARG);

                self.execute(&query, &[&x]).await
            }
            None => {
                let query = format!(cypher_query!(), graph, cypher, CQ_NO_ARG);

                self.execute(&query, &[]).await
            }
        })
    }

    async fn graph_exists(&mut self, name: &str) -> Arc<Result<bool, postgres::Error>> {
        match self.query(GRAPH_EXISTS, &[&name.to_string()]).await {
            Ok(result) => {
                let x: i64 = result[0].get(0);
                return Arc::new(Ok(x == 1));
            }
            Err(e) => return Arc::new(Err(e)),
        }
    }

    async fn prepare_cypher(
        &mut self,
        graph: &str,
        cypher: &str,
        use_arg: bool,
    ) -> Arc<Result<Statement, postgres::Error>> {
        let cypher_arg = if use_arg { CQ_ARG } else { CQ_NO_ARG };
        let query = format!(cypher_query!(), graph, cypher, cypher_arg);

        Arc::new(match self.prepare(&query).await {
            Ok(x) => Ok(x),
            Err(x) => Err(x),
        })
    }
}
