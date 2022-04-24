# Apache AGE (Rust Driver)

## What is Apache AGE

AGE is opensource backend for postgres, that allows user to perform graph related operations on postgres. You can read about it on the [official website](https://age.apache.org/)

## Driver usage

More examples can be find in documentation (link below)

```
 use apache_age::{AgeClient, Client, NoTls, AgType};
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

- [Documentation](https://docs.rs/apache_age/0.1.0/apache_age/)
- [Source code](https://github.com/Dzordzu/rust-apache-age)

