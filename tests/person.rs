#![allow(unused_must_use)]

use apache_age::{AgType, AgeClient, Client, NoTls, Vertex};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

fn connect() -> (Client, String) {
    let mut client =
        Client::connect_age("host=localhost user=postgres password=passwd", NoTls).unwrap();

    let graph_name = "age_test_".to_string()
        + &rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>();

    if let Err(e) = client.create_graph(&graph_name) {
        panic!("{:?}", e);
    }

    (client, graph_name)
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

    client.drop_graph(&graph_name);
}

#[test]
fn person() {
    let (mut client, graph_name) = connect();

    if let Err(e) = client.execute_cypher(
        &graph_name,
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "Alfred".into(),
            surname: "Bohr".into(),
        })),
    ) {
        println!("{:?}", e);
        assert!(false);
    }

    if let Err(e) = client.execute_cypher(
        &graph_name,
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "John".into(),
            surname: "Doe".into(),
        })),
    ) {
        println!("{:?}", e);
        assert!(false);
    }

    match client.query_cypher::<()>(
        &graph_name,
        "MATCH (n: Person) WHERE n.name = 'Alfred' RETURN {name: n.name, surname: n.surname}",
        None
    ) {
        Ok(rows) => {
            let x : AgType<Person> = rows[0].get(0);
            assert_eq!(x.0.surname, "Bohr");

        },
        Err(e) => {
            println!("{:?}", e);
            client.drop_graph(&graph_name);
            panic!();
        },
    }

    client.drop_graph(&graph_name);
}
