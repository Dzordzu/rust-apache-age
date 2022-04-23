use crate::AgType;
use postgres::{Client, Socket, tls::{MakeTlsConnect, TlsConnect}};
use serde::Serialize;

pub trait AgeClient {

    fn connect_age<T>(params: &str, tls_mode: T) -> Result<Client, postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send;

    fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error>;
    fn execute_cypher<T>(
        &mut self,
        name: &str,
        cypher: &str,
        agtype: AgType<T>,
    ) -> Result<u64, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync;
}

impl AgeClient for Client {
    fn create_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute("SELECT * FROM ag_catalog.create_graph($1)", &[&name])
    }

    fn drop_graph(&mut self, name: &str) -> Result<u64, postgres::Error> {
        self.execute("SELECT * FROM ag_catalog.drop_graph($1, true)", &[&name])
    }

    fn execute_cypher<T>(
        &mut self,
        name: &str,
        cypher: &str,
        agtype: AgType<T>,
    ) -> Result<u64, postgres::Error>
    where
        T: Serialize,
        T: std::fmt::Debug,
        T: std::marker::Sync,
    {
        let query: String = "SELECT * FROM ag_catalog.cypher('".to_string()
            + name
            + "',$$ "
            + cypher
            + " $$, $1) AS (v ag_catalog.agtype)";
        self.execute(&query, &[&agtype])
    }

    fn connect_age<T>(params: &str, tls_mode: T) -> Result<Client, postgres::Error>
    where
        T: MakeTlsConnect<Socket> + 'static + Send,
        T::TlsConnect: Send,
        T::Stream: Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send {

        let new_connection = Client::connect(params, tls_mode);

        if let Ok(mut client) =  new_connection {
            /// TODO handle errors
            client.simple_query("CREATE EXTENSION age");
            client.simple_query("LOAD 'age'");
            client.simple_query("SET search_path = ag_catalog, \"$user\", public");
            return Ok(client);
        }
        else { return new_connection; }

    }



}
