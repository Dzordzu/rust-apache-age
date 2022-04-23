#![allow(unused_must_use)]

use apache_age::{AgType, AgeClient, Client, NoTls, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

fn connect() -> (Client, String) {
    let mut client =
        Client::connect_age("host=localhost user=postgres password=passwd", NoTls).unwrap();

    client.drop_graph("age_test");
    client.create_graph("age_test");

    (client, "age_test".into())
}

#[test]
fn simple_query() {
    let (mut client, graph_name) = connect();

    client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &graph_name
            + "', $$ CREATE(n:Person {name: 'T', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
    );

    client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &graph_name
            + "', $$ CREATE(n:Person {name: 'Jack', surname: 'Hell'}) RETURN n $$) AS (v agtype)"),
    );

    // Query, not query_one on puropose, just checking if iterating works
    match client.query(
        &("SELECT v FROM ag_catalog.cypher('".to_string()
            + &graph_name
            + "', $$ MATCH(n: Person) WHERE n.name='T' RETURN n $$) AS (v ag_catalog.agtype)"),
        &[],
    ) {
        Err(e) => {
            print!("{:?}", e);
            assert!(false);
        }
        Ok(query) => {
            let qlen = query.len();
            for row in query {
                let person_vertex: Vertex<Person> = row.get(0);
                assert_eq!(person_vertex.properties().surname, "Doe");
                assert_eq!(person_vertex.properties().name, "T");
            }
            assert_eq!(qlen, 1);
        }
    }

    match client.query(
        &("SELECT v FROM ag_catalog.cypher('".to_string()
            + &graph_name
            + "', $$ MATCH(n: Person) RETURN n $$) AS (v ag_catalog.agtype)"),
        &[],
    ) {
        Err(e) => {
            print!("{:?}", e);
            assert!(false);
        }
        Ok(query) => {
            let qlen = query.len();
            assert_eq!(qlen, 2);
        }
    }
}

#[test]
fn person() {
    let (mut client, graph_name) = connect();

    match client.execute_cypher(
        &graph_name,
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "Alfred".into(),
            surname: "Bohr".into(),
        })),
    ) {
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
        }
        Ok(_) => {}
    }
}
