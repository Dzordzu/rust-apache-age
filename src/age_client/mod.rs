#[macro_use]
mod constants;

#[cfg(feature = "tokio")]
pub mod tokio;

use constants::*;
use crate::AgType;
use postgres::{
    tls::{MakeTlsConnect, TlsConnect},
    Client, Socket,
};
use serde::Serialize;


/// Handles connecting, configuring and querying graph dbs within postgres instance
pub trait AgeClient {
    fn connect_age<T>(params: &str, tls_mode: T) -> Result<Client, postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send;

    /// Create a new constraint for the certain label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str
    ) -> Result<u64, postgres::Error>;

    /// Create unique index for the certain field for the label within graph
    ///
    /// **IMPORTANT**: At least one object has to be created with a certain label
    fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str
    ) -> Result<u64, postgres::Error>;

    fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str
    ) -> Result<u64, postgres::Error>;

    fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    
    /// Exexute cypher query, without any rows to be retured
    fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<u64, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync;

    /// Query cypher for a single agtype (in a format of json)
    ///
    /// **IMPORTANT**: You need to return result of the query as a map
    ///
    /// Example:
    /// ```cypher
    /// MATCH (n: Person) WHERE n.name = 'Alfred' RETURN {name: n.name, surname: n.surname}
    /// ```
    fn query_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<Vec<postgres::Row>, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync;
}

impl AgeClient for Client {
    fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute(CREATE_GRAPH, &[&name])
    }

    fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute(DROP_GRAPH, &[&name])
    }

    fn execute_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<u64, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
    {
        match agtype {
            Some(x) => {
                let query = format!(
                    cypher_query!(),
                    graph, 
                    cypher,
                    CQ_ARG
                );

                self.execute(&query, &[&x])
            }
            None => {
                let query = format!(
                    cypher_query!(),
                    graph, 
                    cypher,
                    CQ_NO_ARG
                );

                self.execute(&query, &[])
            }
        }

    }

    fn connect_age<T>(params: &str, tls_mode: T) -> Result<Client, postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let new_connection = Client::connect(params, tls_mode);

        if let Ok(mut client) = new_connection {
            for query in [
                client.simple_query(LOAD_AGE),
                client.simple_query(SET_AGE),
            ] {
                if let Err(err) = query {
                    return Err(err);
                };
            }
            Ok(client)
        } else {
            new_connection
        }
    }

    fn query_cypher<T>(
        &mut self,
        graph: &str,
        cypher: &str,
        agtype: Option<AgType<T>>,
    ) -> Result<Vec<postgres::Row>, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
    {
        match agtype {
            Some(x) => {
                let query = format!(
                    cypher_query!(),
                    graph, 
                    cypher,
                    CQ_ARG
                );

                self.query(&query, &[&x])
            }
            None => {
                let query = format!(
                    cypher_query!(),
                    graph, 
                    cypher,
                    CQ_NO_ARG
                );

                self.query(&query, &[])
            }
        }
    }

    fn constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        constraint_text: &str
    ) -> Result<u64, postgres::Error> {
        
        let query = format!(
            constraint!(),
            graph,
            label,
            name,
            constraint_text
        );

        self.execute(&query, &[])
    }

    fn unique_index(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str
    ) -> Result<u64, postgres::Error> {
        let query = format!(
            unique_index!(),
            name,
            graph,
            label,
            field
        );

        self.execute(&query, &[])
    }

    fn required_constraint(
        &mut self,
        graph: &str,
        label: &str,
        name: &str,
        field: &str
    ) -> Result<u64, postgres::Error> {
        self.constraint(
            graph, 
            label, 
            name, 
            &format!(
                required_constraint!(), 
                field
            )
        )
    }


}
