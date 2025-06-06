#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::*;
use crate::{map, map_as_slice};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum BinaryFunction {
    Contains,
    StartsWith,
    EndsWith,
    #[cfg(feature = "binary_encoding")]
    HexDecode(bool),
    #[cfg(feature = "binary_encoding")]
    HexEncode,
    #[cfg(feature = "binary_encoding")]
    Base64Decode(bool),
    #[cfg(feature = "binary_encoding")]
    Base64Encode,
    Size,
    #[cfg(feature = "binary_encoding")]
    FromBuffer(DataType, bool),
}

impl BinaryFunction {
    pub(super) fn get_field(&self, mapper: FieldsMapper) -> PolarsResult<Field> {
        use BinaryFunction::*;
        match self {
            Contains => mapper.with_dtype(DataType::Boolean),
            EndsWith | StartsWith => mapper.with_dtype(DataType::Boolean),
            #[cfg(feature = "binary_encoding")]
            HexDecode(_) | Base64Decode(_) => mapper.with_same_dtype(),
            #[cfg(feature = "binary_encoding")]
            HexEncode | Base64Encode => mapper.with_dtype(DataType::String),
            Size => mapper.with_dtype(DataType::UInt32),
            #[cfg(feature = "binary_encoding")]
            FromBuffer(dtype, _) => mapper.with_dtype(dtype.clone()),
        }
    }
}

impl Display for BinaryFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use BinaryFunction::*;
        let s = match self {
            Contains => "contains",
            StartsWith => "starts_with",
            EndsWith => "ends_with",
            #[cfg(feature = "binary_encoding")]
            HexDecode(_) => "hex_decode",
            #[cfg(feature = "binary_encoding")]
            HexEncode => "hex_encode",
            #[cfg(feature = "binary_encoding")]
            Base64Decode(_) => "base64_decode",
            #[cfg(feature = "binary_encoding")]
            Base64Encode => "base64_encode",
            Size => "size_bytes",
            #[cfg(feature = "binary_encoding")]
            FromBuffer(_, _) => "from_buffer",
        };
        write!(f, "bin.{s}")
    }
}

impl From<BinaryFunction> for SpecialEq<Arc<dyn ColumnsUdf>> {
    fn from(func: BinaryFunction) -> Self {
        use BinaryFunction::*;
        match func {
            Contains => {
                map_as_slice!(contains)
            },
            EndsWith => {
                map_as_slice!(ends_with)
            },
            StartsWith => {
                map_as_slice!(starts_with)
            },
            #[cfg(feature = "binary_encoding")]
            HexDecode(strict) => map!(hex_decode, strict),
            #[cfg(feature = "binary_encoding")]
            HexEncode => map!(hex_encode),
            #[cfg(feature = "binary_encoding")]
            Base64Decode(strict) => map!(base64_decode, strict),
            #[cfg(feature = "binary_encoding")]
            Base64Encode => map!(base64_encode),
            Size => map!(size_bytes),
            #[cfg(feature = "binary_encoding")]
            FromBuffer(dtype, is_little_endian) => map!(from_buffer, &dtype, is_little_endian),
        }
    }
}

pub(super) fn contains(s: &[Column]) -> PolarsResult<Column> {
    let ca = s[0].binary()?;
    let lit = s[1].binary()?;
    Ok(ca
        .contains_chunked(lit)?
        .with_name(ca.name().clone())
        .into_column())
}

pub(super) fn ends_with(s: &[Column]) -> PolarsResult<Column> {
    let ca = s[0].binary()?;
    let suffix = s[1].binary()?;

    Ok(ca
        .ends_with_chunked(suffix)?
        .with_name(ca.name().clone())
        .into_column())
}

pub(super) fn starts_with(s: &[Column]) -> PolarsResult<Column> {
    let ca = s[0].binary()?;
    let prefix = s[1].binary()?;

    Ok(ca
        .starts_with_chunked(prefix)?
        .with_name(ca.name().clone())
        .into_column())
}

pub(super) fn size_bytes(s: &Column) -> PolarsResult<Column> {
    let ca = s.binary()?;
    Ok(ca.size_bytes().into_column())
}

#[cfg(feature = "binary_encoding")]
pub(super) fn hex_decode(s: &Column, strict: bool) -> PolarsResult<Column> {
    let ca = s.binary()?;
    ca.hex_decode(strict).map(|ok| ok.into_column())
}

#[cfg(feature = "binary_encoding")]
pub(super) fn hex_encode(s: &Column) -> PolarsResult<Column> {
    let ca = s.binary()?;
    Ok(ca.hex_encode().into())
}

#[cfg(feature = "binary_encoding")]
pub(super) fn base64_decode(s: &Column, strict: bool) -> PolarsResult<Column> {
    let ca = s.binary()?;
    ca.base64_decode(strict).map(|ok| ok.into_column())
}

#[cfg(feature = "binary_encoding")]
pub(super) fn base64_encode(s: &Column) -> PolarsResult<Column> {
    let ca = s.binary()?;
    Ok(ca.base64_encode().into())
}

#[cfg(feature = "binary_encoding")]
pub(super) fn from_buffer(
    s: &Column,
    dtype: &DataType,
    is_little_endian: bool,
) -> PolarsResult<Column> {
    let ca = s.binary()?;
    ca.from_buffer(dtype, is_little_endian)
        .map(|val| val.into())
}

impl From<BinaryFunction> for FunctionExpr {
    fn from(b: BinaryFunction) -> Self {
        FunctionExpr::BinaryExpr(b)
    }
}
