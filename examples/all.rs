use apache_age::sync::{AgeClient, Client};
use apache_age::{AgType, NoTls, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

pub fn main() {
    // Create client
    let mut client = Client::connect_age(
        "host=localhost user=postgres password=passwd port=8081",
        NoTls,
    )
    .unwrap();

    // Create graph
    client.create_graph("my_apache_graph").unwrap();
    assert!(client.graph_exists("my_apache_graph").unwrap());

    // Using a simple postgres query to manipulate graph
    client
        .simple_query(
            &("SELECT * FROM cypher('".to_string()
                + "my_apache_graph"
                + "', $$ "
                + "CREATE(n:Person {name: 'T', surname: 'Doe'}) "
                + "RETURN n "
                + "$$) AS (v agtype)"),
        )
        .unwrap();

    // Using a normal postgres query to operate on graph
    match client.query(
        &("SELECT v FROM ag_catalog.cypher('".to_string()
            + "my_apache_graph"
            + "', $$ "
            + "MATCH(n: Person) WHERE n.name='T' "
            + "RETURN n "
            + "$$) AS (v ag_catalog.agtype)"),
        &[],
    ) {
        Err(_e) => {}
        Ok(query) => {
            for row in query {
                // Vertex usage
                let person_vertex: Vertex<Person> = row.get(0);
                assert_eq!(person_vertex.label(), "Person");
                assert_eq!(person_vertex.properties().surname, "Doe");
            }
        }
    }

    // Using execute_cypher with some input variables
    if let Err(_) = client.execute_cypher(
        "my_apache_graph",
        "CREATE(n: Person {name: $name, surname: $surname})",
        Some(AgType::<Person>(Person {
            name: "John".into(),
            surname: "Doe".into(),
        })),
    ) {
        assert!(false);
    }

    // Using execute_cypher without some input variables
    if let Err(_) = client.execute_cypher::<()>(
        "my_apache_graph",
        "CREATE(n: Person {name: 'Ask', surname: 'Me'})",
        None,
    ) {
        assert!(false);
    }

    // Using query_cypher without parameters
    match client.query_cypher::<()>(
        "my_apache_graph",
        "
            MATCH (n: Person) 
            WHERE n.name = 'Ask' 
            RETURN {name: n.name, surname: n.surname}
        ",
        None,
    ) {
        Ok(rows) => {
            let x: AgType<Person> = rows[0].get(0);
            assert_eq!(x.0.surname, "Me");
        }
        Err(_) => {
            assert!(false);
        }
    }

    // Constraints
    client
        .required_constraint("my_apache_graph", "Person", "myconstraint", "surname")
        .unwrap();

    client
        .unique_index("my_apache_graph", "Person", "myuniq", "name")
        .unwrap();

    assert!(client
        .execute_cypher::<()>(
            "my_apache_graph",
            "CREATE (p: Person { name: 'No surname' })",
            None
        )
        .is_err());

    assert!(client
        .execute_cypher::<()>(
            "my_apache_graph",
            "CREATE (p: Person { name: 'John', surname: 'Repeated name' })",
            None
        )
        .is_err());

    // Drop / destroy graph
    client.drop_graph("my_apache_graph").unwrap();
}
