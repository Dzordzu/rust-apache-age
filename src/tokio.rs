use crate::AgType;
use async_trait::async_trait;
use serde::Serialize;
use tokio_postgres::{
    connect,
    tls::{MakeTlsConnect, TlsConnect},
    Socket,
};

use super::constants::*;

pub use tokio::task::JoinHandle;
pub use tokio_postgres::{Client, Statement, Error};

#[async_trait]
/// Handles connecting, configuring and querying graph dbs within postgres instance
pub trait AgeClient {
    async fn connect_age<T>(
        params: &str,
        tls_mode: T,
    ) -> Result<(Client, JoinHandle<()>), tokio_postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send;

    /// Create a new constraint for the certain label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    async fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str,
    ) -> Result<u64, postgres::Error>;

    /// Create unique index for the certain field for the label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    async fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Result<u64, postgres::Error>;

    async fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Result<u64, postgres::Error>;

    async fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    async fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    async fn graph_exists(&mut self, name: &str) -> Result<bool, postgres::Error>;

    /// Exexute cypher query, without any rows to be retured
    async fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<u64, postgres::Error>
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
    ) -> Result<Vec<postgres::Row>, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send;

    /// Prepare cypher query for future use
    /// ```
    #[doc = include_str!("../examples/prepared_statements_async.rs")]
    /// ```
    async fn prepare_cypher(
        &mut self,
        graph: &str,
        cypher: &str,
        use_arg: bool,
    ) -> Result<Statement, postgres::Error>;
}

#[async_trait]
impl AgeClient for Client {
    async fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute(CREATE_GRAPH, &[&name]).await
    }

    async fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute(DROP_GRAPH, &[&name]).await
    }

    async fn connect_age<T>(
        params: &str,
        tls_mode: T,
    ) -> Result<(Client, JoinHandle<()>), tokio_postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let new_connection = connect(params, tls_mode).await?;

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
                return Err(err);
            };
        }
        Ok((client, handle))
    }

    async fn query_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<Vec<postgres::Row>, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send,
    {
        match agtype {
            Some(x) => {
                let query = format!(cypher_query!(), graph, cypher, CQ_ARG);

                self.query(&query, &[&x]).await
            }
            None => {
                let query = format!(cypher_query!(), graph, cypher, CQ_NO_ARG);

                self.query(&query, &[]).await
            }
        }
    }

    async fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str,
    ) -> Result<u64, postgres::Error> {
        let query = format!(constraint!(), graph, label, name, constraint_text);

        self.execute(&query, &[]).await
    }

    async fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Result<u64, postgres::Error> {
        let query = format!(unique_index!(), name, graph, label, field);

        self.execute(&query, &[]).await
    }

    async fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str,
    ) -> Result<u64, postgres::Error> {
        self.constraint(graph, label, name, &format!(required_constraint!(), field))
            .await
    }

    async fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<u64, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
        T: std::marker::Send,
    {
        match agtype {
            Some(x) => {
                let query = format!(cypher_query!(), graph, cypher, CQ_ARG);

                self.execute(&query, &[&x]).await
            }
            None => {
                let query = format!(cypher_query!(), graph, cypher, CQ_NO_ARG);

                self.execute(&query, &[]).await
            }
        }
    }

    async fn graph_exists(&mut self, name: &str) -> Result<bool, postgres::Error> {
        match self.query(GRAPH_EXISTS, &[&name.to_string()]).await {
            Ok(result) => {
                let x: i64 = result[0].get(0);
                return Ok(x == 1);
            }
            Err(e) => return Err(e),
        }
    }

    async fn prepare_cypher(
        &mut self,
        graph: &str,
        cypher: &str,
        use_arg: bool,
    ) -> Result<Statement, postgres::Error>
    {
        let cypher_arg = if use_arg { CQ_ARG } else { CQ_NO_ARG };
        let query = format!(cypher_query!(), graph, cypher, cypher_arg);

        self.prepare(&query).await
    }
}
