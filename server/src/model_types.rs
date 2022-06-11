use std::string::ToString;

use diesel::query_builder::QueryId;
use diesel::{Expression, expression::NonAggregate, pg::Pg, query_builder::{AstPass, QueryFragment}, sql_types::{self, Text}, sqlite::Sqlite};
use diesel::backend::Backend;
// use diesel::prelude::{Insertable, Queryable};
use ethers::prelude::{U256, H160};
use std::{fmt, io::Write};
use diesel::types::{ToSql, FromSql};
use std::error::Error;
use diesel::serialize::{self, IsNull, Output};
use diesel::deserialize;
use diesel::*;

#[derive(Debug, PartialEq, FromSqlRow)]
pub struct EthereumAddress {
    pub value: H160,
}

impl EthereumAddress {
    pub fn new(value: &H160) -> Self {
        Self { value }
    }
}

impl<'a> From<&'a H160> for EthereumAddress {
    fn from(value: &'a H160) -> Self {
        Self::new(value)
    }
}

impl<'a> Into<&'a H160> for EthereumAddress {
    fn into(self) -> &'a H160 {
        &self.value
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EthereumAddressError {}

impl Error for EthereumAddressError {
    fn description(&self) -> &str {
        "Wrong EthereumAddress field format"
    }
}

impl fmt::Display for EthereumAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'a, DB: Backend> ToSql<Text, DB> for EthereumAddress {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.value.as_bytes().to_sql(out);
        Ok(IsNull::No)
    }
}

impl<'a, DB: Backend> FromSql<Text, DB> for EthereumAddress {
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        if let Some(bytes) = bytes {
            let str = bytes.as_bytes();
            if str.length != 20 {
                Err(Box::new(EthereumAddressError {}))
            } else {
                Ok(H160::from_slice(str))
            }
        }
    }
}

impl Expression for EthereumAddress {
    type SqlType = sql_types::Text;
}

impl NonAggregate for EthereumAddress { }

impl ToString for EthereumAddress {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl QueryId for EthereumAddress {
    type QueryId = Uint256;
    const HAS_STATIC_QUERY_ID: bool = false;
}
