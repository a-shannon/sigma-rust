//! ContextExtension type
use crate::mir::constant::Constant;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use alloc::string::String;
use core::convert::TryFrom;
use core::fmt;
use core::hash::BuildHasher;
use thiserror::Error;

use super::IndexMap;

/// User-defined variables to be put into context
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    feature = "json",
    derive(serde::Deserialize),
    serde(try_from = "IndexMap<String, String>")
)]
pub struct ContextExtension {
    /// key-value pairs of variable id and it's value
    pub values: IndexMap<u8, Constant>,
}

impl ContextExtension {
    /// Returns an empty ContextExtension
    pub fn empty() -> Self {
        Self {
            values: IndexMap::with_hasher(Default::default()),
        }
    }
}

impl fmt::Display for ContextExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.values.iter()).finish()
    }
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.values.len() as u8)?;
        self.values.iter().try_for_each(|(idx, c)| {
            w.put_u8(*idx)?;
            c.sigma_serialize(w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let values_count = r.get_u8()?;
        let mut values: IndexMap<u8, Constant> =
            IndexMap::with_capacity_and_hasher(values_count as usize, Default::default());
        for _ in 0..values_count {
            let idx = r.get_u8()?;
            let value = Constant::sigma_parse(r)?;
            value.tpe.check_v6_type()?;
            values.insert(idx, value);
        }
        Ok(ContextExtension { values })
    }
}

/// Error parsing Constant from base16-encoded string
#[derive(Error, Eq, PartialEq, Debug, Clone)]
#[error("Error parsing constant: {0}")]
pub struct ConstantParsingError(pub String);

// for JSON encoding in ergo-lib
impl<H: BuildHasher> TryFrom<indexmap::IndexMap<String, String, H>> for ContextExtension {
    type Error = ConstantParsingError;
    fn try_from(values_str: indexmap::IndexMap<String, String, H>) -> Result<Self, Self::Error> {
        let values = values_str.iter().try_fold(
            IndexMap::with_capacity_and_hasher(values_str.len(), Default::default()),
            |mut acc, pair| {
                let idx: u8 = pair.0.parse().map_err(|_| {
                    ConstantParsingError(format!("cannot parse index from {0:?}", pair.0))
                })?;
                let constant_bytes = base16::decode(pair.1).map_err(|_| {
                    ConstantParsingError(format!(
                        "cannot decode base16 constant bytes from {0:?}",
                        pair.1
                    ))
                })?;
                acc.insert(
                    idx,
                    Constant::sigma_parse_bytes(&constant_bytes).map_err(|_| {
                        ConstantParsingError(format!(
                            "cannot deserialize constant bytes from {0:?}",
                            pair.1
                        ))
                    })?,
                );
                Ok(acc)
            },
        )?;
        Ok(ContextExtension { values })
    }
}

#[cfg(feature = "json")]
impl serde::Serialize for ContextExtension {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(
                &format!("{}", k),
                &base16::encode_lower(&v.sigma_serialize_bytes().map_err(Error::custom)?),
            )?;
        }
        map.end()
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for ContextExtension {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(
                any::<Constant>().prop_filter(
                    "Filter out types that can't be serialized in ContextExtension",
                    |c| c.tpe.check_v6_type().is_ok(),
                ),
                0..10,
            )
            .prop_map(|constants| {
                let pairs = constants
                    .into_iter()
                    .enumerate()
                    .map(|(idx, c)| (idx as u8, c))
                    .collect();
                Self { values: pairs }
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{serialization::sigma_serialize_roundtrip, unsignedbigint256::UnsignedBigInt};
    use proptest::prelude::*;

    #[test]
    #[should_panic]
    fn test_v6_type_reject() {
        let mut extension = ContextExtension::empty();
        extension
            .values
            .insert(0, Constant::from(UnsignedBigInt::from(1u32)));
        sigma_serialize_roundtrip(&extension);
    }

    proptest! {
        #[test]
        fn ser_roundtrip(v in any::<ContextExtension>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
    #[cfg(feature = "json")]
    mod json {
        use super::*;
        #[test]
        fn parse_empty_context_extension() {
            let c: ContextExtension = serde_json::from_str("{}").unwrap();
            assert_eq!(c, ContextExtension::empty());
        }

        #[test]
        fn parse_context_extension() {
            let json = r#"
            {"1" :"05b0b5cad8e6dbaef44a", "3":"048ce5d4e505"}
            "#;
            let c: ContextExtension = serde_json::from_str(json).unwrap();
            assert_eq!(c.values.len(), 2);
            assert!(c.values.get(&1u8).is_some());
            assert!(c.values.get(&3u8).is_some());
        }
    }
}
