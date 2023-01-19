use bytes::BufMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Represents vertex within graph. Used during process of vertex deserialization
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

/// Represents edge within graph. Used during process of edge deserialization
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Path<V, E> {
    vertices: Vec<Vertex<V>>,
    edges: Vec<Edge<E>>,
}

impl<V, E> Path<V, E> {
    pub fn vertices(&self) -> &Vec<Vertex<V>> {
        &self.vertices
    }

    pub fn edges(&self) -> &Vec<Edge<E>> {
        &self.edges
    }
}

const VERTEX_SUFFIX: &[u8] = "::vertex".as_bytes();
const EDGE_SUFFIX: &[u8] = "::edge".as_bytes();
const PATH_SUFFIX: &[u8] = "::path".as_bytes();

const VERTEX_SUFFIX_LEN: usize = 8;
const EDGE_SUFFIX_LEN: usize = 6;
const PATH_SUFFIX_LEN: usize = 6;

impl<'a, T> FromSql<'a> for Vertex<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(
        ty: &Type,
        mut raw: &'a [u8],
    ) -> Result<Vertex<T>, Box<dyn std::error::Error + Sync + Send>> {
        if ty.schema() != "ag_catalog" || ty.name() != "agtype" {
            return Err("Only ag_catalog.agtype is supported".into());
        }

        let mut b = [0; 1];
        raw.read_exact(&mut b)?;

        // We only support version 1 of the jsonb binary format
        if b[0] != 1 {
            return Err("unsupported JSONB encoding version".into());
        }

        // Remove ::vertex from bytes
        let raw_splitted = raw.split_at(raw.len() - VERTEX_SUFFIX_LEN).0;

        serde_json::de::from_slice::<Vertex<T>>(raw_splitted).map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

impl<'a, T> FromSql<'a> for Edge<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(
        ty: &Type,
        mut raw: &'a [u8],
    ) -> Result<Edge<T>, Box<dyn std::error::Error + Sync + Send>> {
        if ty.schema() != "ag_catalog" || ty.name() != "agtype" {
            return Err("Only ag_catalog.agtype is supported".into());
        }

        let mut b = [0; 1];
        raw.read_exact(&mut b)?;

        // We only support version 1 of the jsonb binary format
        if b[0] != 1 {
            return Err("unsupported JSONB encoding version".into());
        }

        // Remove ::edge from bytes
        let raw_splitted = raw.split_at(raw.len() - EDGE_SUFFIX_LEN).0;

        serde_json::de::from_slice::<Edge<T>>(raw_splitted).map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

/// Represents path in graph. Used during process of path deserialization
impl<'a, V, E> FromSql<'a> for Path<V, E>
where
    V: Deserialize<'a>,
    E: Deserialize<'a>,
{
    fn from_sql(
        ty: &Type,
        mut raw: &'a [u8],
    ) -> Result<Path<V, E>, Box<dyn std::error::Error + Sync + Send>> {
        if ty.schema() != "ag_catalog" || ty.name() != "agtype" {
            return Err("Only ag_catalog.agtype is supported".into());
        }

        let mut b = [0; 1];
        raw.read_exact(&mut b)?;

        // We only support version 1 of the jsonb binary format
        if b[0] != 1 {
            return Err("unsupported JSONB encoding version".into());
        }

        if !(raw[0] == "[".as_bytes()[0] && &raw[raw.len() - PATH_SUFFIX_LEN..] == PATH_SUFFIX) {
            return Err("Invalid path definition".into());
        }

        let mut vertices: Vec<Vertex<V>> = vec![];
        let mut edges: Vec<Edge<E>> = vec![];

        let mut first_open_bracket = raw.len();

        for (i, character) in raw[..raw.len() - PATH_SUFFIX_LEN - 1].iter().enumerate() {
            if *character as char == '{' && first_open_bracket == raw.len() {
                first_open_bracket = i;
            } else if &raw[i..i + VERTEX_SUFFIX_LEN] == VERTEX_SUFFIX {
                match serde_json::de::from_slice::<Vertex<V>>(&raw[first_open_bracket..i]) {
                    Ok(vertex) => {
                        vertices.push(vertex);
                        first_open_bracket = raw.len();
                    }
                    Err(e) => return Err(e.into()),
                };
            } else if &raw[i..i + EDGE_SUFFIX_LEN] == EDGE_SUFFIX {
                match serde_json::de::from_slice::<Edge<E>>(&raw[first_open_bracket..i]) {
                    Ok(edge) => {
                        edges.push(edge);
                        first_open_bracket = raw.len();
                    }
                    Err(e) => return Err(e.into()),
                };
            }
        }
        Ok(Path { vertices, edges })
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

/// Simple wrapper (similar to JSONB) that handles agtype serialization and deserialization
#[derive(Debug, Serialize, Deserialize)]
pub struct AgType<T>(pub T);

impl<T> ToSql for AgType<T>
where
    T: Serialize,
    T: std::fmt::Debug,
{
    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }

    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        out.put_u8(1);
        serde_json::ser::to_writer(out.writer(), &self.0)?;
        Ok(IsNull::No)
    }

    to_sql_checked!();
}

impl<'a, T> FromSql<'a> for AgType<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(
        ty: &Type,
        mut raw: &'a [u8],
    ) -> Result<AgType<T>, Box<dyn std::error::Error + Sync + Send>> {
        if ty.schema() != "ag_catalog" || ty.name() != "agtype" {
            return Err("Only ag_catalog.agtype is supported".into());
        }

        let mut b = [0; 1];
        raw.read_exact(&mut b)?;

        // We only support version 1 of the jsonb binary format
        if b[0] != 1 {
            return Err("unsupported JSONB encoding version".into());
        }

        serde_json::de::from_slice::<AgType<T>>(raw).map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}
