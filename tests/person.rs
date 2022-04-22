use apache_age::{AgType, AgeClient, Client, NoTls, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    pub name: String,
    pub surname: String,
}

#[test]
fn person() {
    let mut client =
        Client::connect("host=localhost user=postgres password=passwd", NoTls).unwrap();

    client.drop_graph("age_test");
    client.create_graph("age_test");

    // match client.execute_cypher(
    //     "age_test",
    //     "CREATE(n: Person {name: $name, surname: $surname})",
    //     AgType::<Person>(Person {
    //         name: "Alfred".into(),
    //         surname: "Bohr".into(),
    //     }),
    // ) {
    //     Err(e) => {
    //         println!("{:?}", e);
    //         assert_eq!(1, 2);
    //     }
    //     Ok(_) => {}
    // }

    match client.query(
        "SELECT v FROM ag_catalog.cypher('age', $$ MATCH(n: Person) WHERE 1=1 RETURN n $$) AS (v ag_catalog.agtype);",
        &[],
    ) {
        Err(e) => {
            print!("{:?}", e);
            assert!(false);
        }
        Ok(query) => {
            let qlen = query.len();
            for row in query {
                let x: Result<Vertex<Person>, postgres::Error> = row.try_get(0);
                match x {
                    Err(e) => println!("Err: {:?}", e),
                    Ok(obj) => {
                        println!("{:?}", obj);
                    }
                }
            }
            assert_eq!(qlen, 1221);
        }
    }
}
