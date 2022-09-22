use apache_age::sync::{AgeClient, Client};
use apache_age::{AgType, NoTls, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Day<'a> {
    pub name: &'a str,
    pub is_rainy: bool,
    pub month: u8,
}

unsafe impl<'a> Sync for Day<'a> {}

pub fn main() {
    let mut client = Client::connect_age(
        "host=localhost user=postgres password=passwd port=8081",
        NoTls,
    )
    .unwrap();

    client.create_graph("prepared_statementes_sync").unwrap();

    let statement = client
        .prepare_cypher(
            "prepared_statementes_sync",
            "CREATE (x: PreparedDay { name: $name, is_rainy: $is_rainy, month: $month })",
            true,
        )
        .unwrap();

    let day = Day {
        name: "Some day",
        is_rainy: false,
        month: 2,
    };

    client.query(&statement, &[&AgType(day)]).unwrap();

    match client.query_cypher::<()>(
        "prepared_statementes_sync",
        "MATCH (x: PreparedDay) RETURN x",
        None,
    ) {
        Ok(x) => {
            let day: Vertex<Day> = x[0].get(0);
            assert_eq!(day.properties().month, 2);
            assert_eq!(day.properties().is_rainy, false);
            assert_eq!(day.properties().name, "Some day");
        }
        Err(_) => {
            assert!(false)
        }
    }

    client.drop_graph("prepared_statementes_sync").unwrap();
}
