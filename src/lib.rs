mod age_client;
mod structures;

pub use age_client::AgeClient;
pub use postgres::{Client, NoTls};
pub use structures::{Edge, Vertex};

use bytes::BufMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Read;

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

        serde_json::de::from_slice::<Edge<T>>(raw_splitted).map_err(Into::into)
    }

    fn accepts(ty: &Type) -> bool {
        ty.schema() == "ag_catalog" && ty.name() == "agtype"
    }
}

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
    ) -> Result<postgres_types::IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        out.put_u8(1);
        serde_json::ser::to_writer(out.writer(), &self.0)?;
        Ok(IsNull::No)
    }

    to_sql_checked!();
}
