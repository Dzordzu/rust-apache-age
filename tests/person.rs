#![allow(unused_must_use)]

use apache_age::{AgType, AgeClient, Client, NoTls, Vertex};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

struct TestConnection {
    pub client: Client,
    pub graph_name: String,
}

impl TestConnection {
    pub fn new() -> Self {
        let (client, graph_name) = connect();

        Self { 
            client, 
            graph_name

        }
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

    if let Err(e) = client.create_graph(&graph_name) {
        println!("{:?}", e);
        assert!(false);
    }

    (client, graph_name)
}

#[test]
fn simple_query() {
    let mut tc = TestConnection::new();

    tc.client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &tc.graph_name
            + "', $$ CREATE(n:Person {name: 'T', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
    );

    tc.client.simple_query(
        &("SELECT * FROM cypher('".to_string()
            + &tc.graph_name
            + "', $$ CREATE(n:Person {name: 'Jack', surname: 'Hell'}) RETURN n $$) AS (v agtype)"),
    );

    // Query, not query_one on puropose, just checking if iterating works
    match tc.client.query(
        &("SELECT v FROM ag_catalog.cypher('".to_string()
            + &tc.graph_name
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

    match tc.client.query(
        &("SELECT v FROM ag_catalog.cypher('".to_string()
            + &tc.graph_name
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
    let mut tc = TestConnection::new();

    match tc.client.graph_exists(&tc.graph_name) {
        Ok(r) => {
            assert!(r);
        },
        Err(_) => assert!(false),
    }

    if let Err(e) = tc.client.execute_cypher(
        &tc.graph_name,
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "Alfred".into(),
            surname: "Bohr".into(),
        })),
    ) {
        println!("{:?}", e);
        assert!(false);
    }

    if let Err(e) = tc.client.execute_cypher(
        &tc.graph_name,
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "John".into(),
            surname: "Doe".into(),
        })),
    ) {
        println!("{:?}", e);
        assert!(false);
    }

    match tc.client.query_cypher::<()>(
        &tc.graph_name,
        "MATCH (n: Person) WHERE n.name = 'Alfred' RETURN {name: n.name, surname: n.surname}",
        None,
    ) {
        Ok(rows) => {
            let x: AgType<Person> = rows[0].get(0);
            assert_eq!(x.0.surname, "Bohr");
        }
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IdPassing {
    id: usize
}

#[test]
fn unique_index() {
    let mut tc = TestConnection::new();

    let result = tc.client.query_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {name: 'Dummy', surname: 'Name'}) RETURN id(n)",
        None
    );

    let dummy_person : usize = match result {
        Ok(rows) => { rows[0].get(0) }
        Err(_) => { 
            assert!(false); 
            AgType::<usize>(0)
        }
    }.0;

    tc.client.unique_index(
        &tc.graph_name,
        "Person",
        "myconstraint",
        "surname"
    );


    tc.client.execute_cypher::<IdPassing>(
        &tc.graph_name, 
        "MATCH (n: Person) WHERE id(n) = $id DELETE n",
        Some(AgType::<IdPassing>(IdPassing { id: dummy_person }))
    );

    tc.client.execute_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {surname: 'Name'})",
        None
    );

    match tc.client.execute_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {name: 'Dummy', surname: 'Name'})",
        None
    ) {
        Ok(_) => {
            println!("One must not be able to perform this operation");
            assert!(false);
        }
        Err(_) => {
        }
    }

    match tc.client.execute_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {surname: 'Name'})",
        None
    ) {
        Ok(_) => {
            println!("One must not be able to perform this operation");
            assert!(false);
        }
        Err(_) => {
        }
    }
}

#[test]
fn required_constraint() {
    let mut tc = TestConnection::new();

    let result = tc.client.query_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {name: 'Dummy', surname: 'Name'}) RETURN id(n)",
        None
    );

    let dummy_person : usize = match result {
        Ok(rows) => { rows[0].get(0) }
        Err(_) => { 
            assert!(false); 
            AgType::<usize>(0)
        }
    }.0;

    tc.client.required_constraint(
        &tc.graph_name,
        "Person",
        "myconstraint",
        "surname"
    );

    tc.client.execute_cypher::<IdPassing>(
        &tc.graph_name, 
        "MATCH (n: Person) WHERE id(n) = $id DELETE n",
        Some(AgType::<IdPassing>(IdPassing { id: dummy_person }))
    );


    match tc.client.execute_cypher::<()>(
        &tc.graph_name,
        "CREATE(n: Person {name: 'Name'})",
        None
    ) {
        Ok(_) => {
            println!("One must not be able to perform this operation");
            assert!(false);
        }
        Err(_) => {}
    }


}
