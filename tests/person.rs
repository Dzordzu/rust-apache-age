use apache_age::{AgType, AgeClient, Client, NoTls, Vertex};
use postgres::{SimpleQueryMessage, SimpleQueryRow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    pub name: String,
    pub surname: String,
}

#[test]
fn person() {
    let mut client =
        Client::connect_age("host=localhost user=postgres password=passwd", NoTls).unwrap();

    client.drop_graph("age");
    client.create_graph("age");

    client.simple_query("SELECT * FROM cypher('age', $$ CREATE(n:Person {name: 'T', surname: 'Doe'}) RETURN n $$) AS (v agtype)");

    match client.query(
        "SELECT v FROM ag_catalog.cypher('age', $$ MATCH(n: Person) WHERE 1=1 RETURN n $$) AS (v ag_catalog.agtype);", &[]
    ) {
        Err(e) => {
            print!("{:?}", e);
            assert!(false);
        }
        Ok(query) => {
            let qlen = query.len();
            for row in query {
                let person_vertex : Vertex<Person> = row.get(0);
            }
            assert_eq!(qlen, 1);
        }
    }

    match client.execute_cypher(
        "age",
        "CREATE(n: Person {name: $name, surname: $surname})",
        AgType::<Person>(Person {
            name: "Alfred".into(),
            surname: "Bohr".into(),
        }),
    ) {
        Err(e) => {
            println!("{:?}", e);
            assert!(false);
        }
        Ok(_) => {}
    }

}
