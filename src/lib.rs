mod age_client;
mod structures;

pub use age_client::AgeClient;
pub use postgres::{Client, NoTls};

use bytes::BufMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vertex<T>(pub crate::structures::Vertex<T>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge<T>(pub crate::structures::Edge<T>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgType<T>(pub T);

impl<'a, T> FromSql<'a> for Vertex<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Vertex<T>, Box<dyn Error + Sync + Send>> {
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
        let raw_splitted = raw.split_at(raw.len() - 8).0;

        serde_json::de::from_slice(raw_splitted)
            .map(Vertex)
            .map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

impl<'a, T> FromSql<'a> for Edge<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Edge<T>, Box<dyn Error + Sync + Send>> {
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
        let raw_splitted = raw.split_at(raw.len() - 6).0;

        serde_json::de::from_slice(raw_splitted)
            .map(Edge)
            .map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

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
        ty: &Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        let serialized_json = serde_json::to_string(&self.0).unwrap();
        let arg = String::from("'") + &serialized_json + "'";
        println!("{}", arg);
        println!("{:?}", ty);
        out.writer().write_all(arg.as_bytes());
        Ok(IsNull::No)
    }

    to_sql_checked!();
}
