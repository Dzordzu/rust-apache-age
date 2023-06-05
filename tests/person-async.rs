#![cfg(feature = "tokio")]
#![allow(unused_must_use)]

use apache_age::tokio::{AgeClient, Client, JoinHandle};
use apache_age::{NoTls, Vertex};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

const CONN: &str = "host=localhost user=postgres password=passwd port=8081";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

struct TestConnection {
    pub client: Client,
    pub graph_name: String,
    join_handle: JoinHandle<()>,
}

impl TestConnection {
    pub async fn new() -> Self {
        let (client, join_handle, graph_name) = connect().await;

        Self {
            client,
            join_handle,
            graph_name,
        }
    }
}

impl Drop for TestConnection {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

async fn connect() -> (Client, JoinHandle<()>, String) {
    let (mut client, join_handle) = Client::connect_age(CONN, NoTls).await.unwrap();

    let graph_name = "age_test_".to_string()
        + &rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>();

    assert!(client.create_graph(&graph_name).await.is_ok());

    (client, join_handle, graph_name)
}

#[tokio::test]
async fn simple_query() {
    let mut tc = TestConnection::new().await;

    tc.client
        .simple_query(
            &("SELECT * FROM cypher('".to_string()
                + &tc.graph_name
                + "', $$ CREATE(n:Person {name: 'T', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
        )
        .await;

    tc.client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &tc.graph_name
            + "', $$ CREATE(n:Person {name: 'Jack', surname: 'Hell'}) RETURN n $$) AS (v agtype)"),
    ).await;

    // Query, not query_one on puropose, just checking if iterating works
    let query = tc
        .client
        .query(
            &("SELECT v FROM ag_catalog.cypher('".to_string()
                + &tc.graph_name
                + "', $$ MATCH(n: Person) WHERE n.name='T' RETURN n $$) AS (v ag_catalog.agtype)"),
            &[],
        )
        .await
        .unwrap();
    let qlen = query.len();
    for row in query {
        let person_vertex: Vertex<Person> = row.get(0);
        assert_eq!(person_vertex.properties().surname, "Doe");
        assert_eq!(person_vertex.properties().name, "T");
    }
    assert_eq!(qlen, 1);

    let query = tc
        .client
        .query(
            &("SELECT v FROM ag_catalog.cypher('".to_string()
                + &tc.graph_name
                + "', $$ MATCH(n: Person) RETURN n $$) AS (v ag_catalog.agtype)"),
            &[],
        )
        .await
        .unwrap();
    let qlen = query.len();
    assert_eq!(qlen, 2);

    assert!(tc.client.graph_exists(&tc.graph_name).await.unwrap());

    tc.client.drop_graph(&tc.graph_name).await;
}
