use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vertex<T> {
    id: i64,
    label: String,
    properties: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge<T> {
    id: i64,
    label: String,
    properties: T,
    start_id: i64,
    end_id: i64,
}
