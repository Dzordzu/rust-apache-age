use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Vertex<T> {
    id: u64,
    label: String,
    properties: T,
}

impl<T> Vertex<T> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn properties(&self) -> &T {
        &self.properties
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge<T> {
    id: u64,
    label: String,
    properties: T,
    start_id: u64,
    end_id: u64,
}

impl<T> Edge<T> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn properties(&self) -> &T {
        &self.properties
    }

    pub fn start_id(&self) -> u64 {
        self.start_id
    }

    pub fn end_id(&self) -> u64 {
        self.end_id
    }
}
