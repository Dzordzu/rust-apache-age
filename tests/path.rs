#![cfg(feature = "sync")]
#![allow(unused_must_use)]

use apache_age::sync::{AgeClient, Client};
use apache_age::{NoTls, Path};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChildOf {
    pub surname: String,
}

struct TestConnection {
    pub client: Client,
    pub graph_name: String,
}

impl TestConnection {
    pub fn new() -> Self {
        let (client, graph_name) = connect();

        Self { client, graph_name }
    }
}

impl Drop for TestConnection {
    fn drop(&mut self) {
        self.client.drop_graph(&self.graph_name);
    }
}

fn connect() -> (Client, String) {
    let mut client = Client::connect_age(
        "host=localhost user=postgres password=passwd port=8081",
        NoTls,
    )
    .unwrap();

    let graph_name = "age_test_".to_string()
        + &rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>();

    assert!(client.create_graph(&graph_name).is_ok());

    (client, graph_name)
}

#[test]
fn test_path() {
    let mut tc: TestConnection = TestConnection::new();

    tc.client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &tc.graph_name
            + "', $$ CREATE(n:Person {name: 'John', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
    );

    tc.client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &tc.graph_name
            + "', $$ CREATE(n:Person {name: 'Jane', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
    );

    let query = "
    SELECT v FROM ag_catalog.cypher('"
        .to_string()
        + &tc.graph_name
        + "',
        $$
            MATCH (a:Person), (b:Person)
            WHERE a.name = 'Jane' AND b.name = 'John'
            CREATE (a)-[e:ChildOf { surname:a.surname }]->(b)
        $$) as (v agtype)
    ";

    tc.client.query(&query, &[]).unwrap();

    let query = tc
        .client
        .query(
            &("SELECT v FROM ag_catalog.cypher('".to_string()
                + &tc.graph_name
                + "', $$ MATCH p=()-->() return p $$) AS (v ag_catalog.agtype)"),
            &[],
        )
        .unwrap();
    let qlen = query.len();
    assert_eq!(qlen, 1);
    for row in query {
        let path: Path<Person, ChildOf> = row.get(0);
        assert_eq!(path.vertices().len(), 2);
        assert_eq!(path.edges().len(), 1);

        let v1 = &path.vertices()[0];
        assert_eq!(v1.label(), "Person");
        assert_eq!(v1.properties().name, "Jane");
        assert_eq!(v1.properties().surname, "Doe");

        let v2 = &path.vertices()[1];
        assert_eq!(v2.label(), "Person");
        assert_eq!(v2.properties().name, "John");
        assert_eq!(v2.properties().surname, "Doe");

        let e = &path.edges()[0];
        assert_eq!(e.label(), "ChildOf");
        assert_eq!(e.properties().surname, "Doe");
        assert_eq!(e.start_id(), v1.id());
        assert_eq!(e.end_id(), v2.id());
    }
}
