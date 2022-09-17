# Apache AGE (Rust Driver)

## What is Apache AGE

AGE is opensource backend for postgres, that allows user to perform graph related operations on postgres. You can read about it on the [official website](https://age.apache.org/)

This repository will be eventually merged into the [age repository](https://github.com/apache/age). The status of the work needed for PR can be found in [the special issue](https://github.com/apache/age/issues/262) within AGE issue tracker

## Driver usage

More examples can be find in documentation (link below)

```rust
use apache_age::{NoTls, AgType};
use apache_age::sync::{AgeClient, Client}; 
use serde::{Deserialize, Serialize};

let mut client = Client::connect_age(
  "host=localhost user=postgres password=passwd port=8081",
  NoTls
).unwrap();

client.create_graph("my_apache_graph");

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    pub name: String,
    pub surname: String,
}

match client.query_cypher::<()>(
    "my_apache_graph",
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

client.drop_graph("my_apache_graph");
```

## Links

- [Documentation](https://docs.rs/apache_age/latest/apache_age/)
- [Source code](https://github.com/Dzordzu/rust-apache-age)

## Testing

There is a simple docker-compose file within tests directory. Run it to set up an AGE db.

```bash
pushd tests
docker-compose up -d
popd
cargo t
```
