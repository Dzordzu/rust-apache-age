use crate::AgType;
use postgres::Client;
use serde::Serialize;

pub trait AgeClient {
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
}
