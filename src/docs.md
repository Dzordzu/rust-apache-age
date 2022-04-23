Library that can be used as a connector for the apache age instance. Uses prepared statements by default

# Examples

You can see more within `tests` directory

## Connecting

```rust
   use apache_age::{AgeClient, Client, NoTls};
   let mut client = Client::connect_age(
     "host=localhost user=postgres password=passwd port=8081",
     NoTls
   ).unwrap();
```

## Creating/destroying graphs
```rust
   client.create_graph("my_apache_graph");
   client.drop_graph("my_apache_graph");
```

## Using simple postgres queries
```rust
   client.simple_query(
       &("SELECT * FROM cypher('".to_string()
           + &graph_name
           + "', $$ CREATE(n:Person {name: 'T', surname: 'Doe'}) RETURN n $$) AS (v agtype)"),
   );

```

## Reading rows using simple postgres queries
```rust
   match client.query(
       &("SELECT v FROM ag_catalog.cypher('".to_string()
           + &graph_name
           + "', $$ MATCH(n: Person) WHERE n.name='T' RETURN n $$) AS (v ag_catalog.agtype)"),
       &[],
   ) {
       Err(e) => {
           // Handle error
       }
       Ok(query) => {
           for row in query {
               let person_vertex: Vertex<Person> = row.get(0);
           }
           assert_eq!(qlen, 1);
       }
   }

```

## Using cypher execute method

With parameters
```rust
   if let Err(e) = client.execute_cypher(
       &graph_name,
       "CREATE(n: Person {name: $name, surname: $surname})",
       Some(AgType::<Person>(Person {
          // Here you pass your agtype
           name: "John".into(),
           surname: "Doe".into(),
       })),
   ) {
      // Handle error
   }
```

And without
```rust
   if let Err(e) = client.execute_cypher::<()>(
       &graph_name,
       "CREATE(n: Person {name: 'Ask', surname: 'Me'})",
       None
   ) {
      // Handle error
   }
```

## Querying cypher

```rust
    match client.query_cypher::<()>(
        &graph_name,
        "MATCH (n: Person) WHERE n.name = 'Alfred' RETURN {name: n.name, surname: n.surname}",
        None,
    ) {
        Ok(rows) => {
            let x: AgType<Person> = rows[0].get(0);
            // do whatever you need
        }
        Err(e) => {
            // handle error
        }
    }
```
