use apache_age::tokio::{AgeClient, Client};
use apache_age::{AgType, NoTls, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Day<'a> {
    pub name: &'a str,
    pub is_rainy: bool,
    pub month: u8,
}

unsafe impl<'a> Sync for Day<'a> {}

#[tokio::main]
pub async fn main() {
    let (client, _) = Client::connect_age(
        "host=localhost user=postgres password=passwd port=8081",
        NoTls,
    )
    .await
    .unwrap();

    client.create_graph("prepared_statements").await.unwrap();

    let statement = client
        .prepare_cypher(
            "prepared_statements",
            "CREATE (x: PreparedDay { name: $name, is_rainy: $is_rainy, month: $month })",
            true,
        )
        .await
        .unwrap();

    let day = Day {
        name: "Some day",
        is_rainy: false,
        month: 2,
    };

    client.query(&statement, &[&AgType(day)]).await.unwrap();

    let x = client
        .query_cypher::<()>(
            "prepared_statements",
            "MATCH (x: PreparedDay) RETURN x",
            None,
        )
        .await
        .unwrap();

    let day: Vertex<Day> = x[0].get(0);
    assert_eq!(day.properties().month, 2);
    assert!(!day.properties().is_rainy);
    assert_eq!(day.properties().name, "Some day");

    client.drop_graph("prepared_statements").await.unwrap();
}
