use core::fmt::Formatter;
use core::hash::Hash;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::mir::expr::InvalidArgumentError;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    name_bytes: Vec<u8>,
}

impl core::fmt::Debug for STypeVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.as_string().fmt(f)
    }
}

impl STypeVar {
    /// Creates a type variable from a UTF8 text string (name length is a `u8`, so 0..=255 bytes)
    pub fn new_from_str(name: &'static str) -> Result<Self, InvalidArgumentError> {
        Ok(Self {
            name_bytes: name.to_string().into_bytes(),
        })
    }

    /// Creates a type variable from bytes of a UTF8 text string (name length 0..=255).
    ///
    /// Mirrors the JVM `TypeSerializer` (`new String(bytes, UTF_8)`): a non-UTF8 name is
    /// lossily decoded -- malformed bytes become U+FFFD -- and stored canonicalized, rather
    /// than rejected. (`Result` is kept for API stability; this no longer errors.)
    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, InvalidArgumentError> {
        Ok(Self {
            name_bytes: String::from_utf8_lossy(&bytes).into_owned().into_bytes(),
        })
    }

    /// Returns text representation (e.g "T", etc.)
    pub fn as_string(&self) -> String {
        #[allow(clippy::unwrap_used)]
        String::from_utf8(self.name_bytes.clone()).unwrap()
    }

    /// "T" type variable
    pub fn t() -> Self {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("T").unwrap()
    }

    /// "IV"(Input Value) type variable
    pub fn iv() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("IV").unwrap()
    }
    /// "OV"(Output Value) type variable
    pub fn ov() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("OV").unwrap()
    }
}

impl SigmaSerializable for STypeVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.name_bytes.len() as u8)?;
        // name length is one byte (PutByteCost) -- Scala `TypeSerializer`'s `putUByte`
        w.add_put_byte_cost();
        w.write_all(self.name_bytes.as_slice())?;
        // name bytes are written as one block -- Scala `putBytes` => PutChunkCost over the length
        w.add_put_chunk_cost(self.name_bytes.len());
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let name_len = r.get_u8()?;
        let mut bytes = vec![0; name_len as usize];
        r.get_bytes_into(&mut bytes)?;
        Ok(STypeVar::new_from_bytes(bytes)?)
    }
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    pub(crate) ident: STypeVar,
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for STypeVar {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            "[A-Z][A-Z0-9]{0,2}"
                .prop_map(|name| STypeVar::new_from_bytes(name.into_bytes()).unwrap())
                .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // The JVM `TypeSerializer.deserialize` reads the type-var name length via
    // `getUByte()` (0..=255, no bound) then `new String(getBytes(len))`, so an
    // empty name (len 0) and a 255-byte name both deserialize. The previous
    // `BoundedVec<u8, 1, 254>` over-rejected both -- a divergence from sigma-state.
    // See sigma-state core/.../serialization/TypeSerializer.scala:202-206.

    #[test]
    fn parse_name_length_0() {
        // u8 length 0x00, zero name bytes -> STypeVar("")
        let tv = STypeVar::sigma_parse_bytes(&[0x00u8]).unwrap();
        assert_eq!(tv.as_string(), "");
        assert_eq!(tv.sigma_serialize_bytes().unwrap(), vec![0x00u8]);
    }

    #[test]
    fn parse_name_length_255() {
        // u8 length 0xff (255), then 255 'a' bytes -> STypeVar("a" * 255)
        let mut bytes = vec![0xffu8];
        bytes.extend_from_slice(&[b'a'; 255]);
        let tv = STypeVar::sigma_parse_bytes(&bytes).unwrap();
        assert_eq!(tv.as_string(), "a".repeat(255));
        assert_eq!(tv.sigma_serialize_bytes().unwrap(), bytes);
    }

    #[test]
    fn parse_non_utf8_name_lossy() {
        // The JVM does `new String(bytes, UTF_8)` -- lossy. A non-UTF8 name parses to the
        // U+FFFD-canonicalized form instead of erroring. Here: u8 length 1, byte 0xff.
        let tv = STypeVar::sigma_parse_bytes(&[0x01u8, 0xff]).unwrap();
        assert_eq!(tv.as_string(), "\u{fffd}");
        // re-serializes as the canonical UTF-8 of U+FFFD (ef bf bd), length 3
        assert_eq!(
            tv.sigma_serialize_bytes().unwrap(),
            vec![0x03u8, 0xef, 0xbf, 0xbd]
        );
    }
}
