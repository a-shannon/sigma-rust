use super::op_code::OpCode;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SigmaSerializationError;
use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::types::stuple;
use crate::types::stype::SType;
use alloc::string::ToString;
use alloc::vec::Vec;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)] // to differentiate from similarly named SType enum variants
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum TypeCode {
    SBOOLEAN = 1,
    SBYTE = 2,
    SSHORT = 3,
    SINT = 4,
    SLONG = 5,
    SBIGINT = 6,
    SGROUP_ELEMENT = 7,
    SSIGMAPROP = 8,
    SUNSIGNEDBIGINT = 9,
    COLL = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::COLLECTION_CONSTR_ID, // 12 * 1
    NESTED_COLL = (TypeCode::MAX_PRIM_TYPECODE + 1) * (2 * TypeCode::COLLECTION_CONSTR_ID), // 12 * 2 = 24
    OPTION = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_CONSTR_ID, // 12 * 3 = 36
    OPTION_COLL = (TypeCode::MAX_PRIM_TYPECODE + 1)
        * (TypeCode::COLLECTION_CONSTR_ID + TypeCode::OPTION_CONSTR_ID), // 12 * 4 = 48

    TUPLE_PAIR1 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR1_CONSTR_ID, // 12 * 5 = 60
    TUPLE_PAIR2 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR2_CONSTR_ID, // 72
    TUPLE_PAIR_SYMMETRIC =
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID, // 84
    TUPLE = (TypeCode::MAX_PRIM_TYPECODE + 1) * 8, // 12 * 8 = 96
    SANY = 97,
    SUNIT = 98,
    SBOX = 99,
    SAVL_TREE = 100,
    SCONTEXT = 101,
    SSTRING = 102,
    STYPE_VAR = 103,
    SHEADER = 104,
    SPRE_HEADER = 105,
    SGLOBAL = 106,
}

impl TypeCode {
    /// SFunc types occupy remaining space of byte values [FirstFuncType .. 255]
    #[allow(dead_code)]
    const FIRST_FUNC_TYPE: u8 = OpCode::LAST_DATA_TYPE.value();
    #[allow(dead_code)]
    const LAST_FUNC_TYPE: u8 = 255;

    /// Type code of the last valid prim type so that (1 to LastPrimTypeCode) is a range of valid codes.
    #[allow(dead_code)]
    const LAST_PRIM_TYPECODE: u8 = 8;

    /// Upper limit of the interval of valid type codes for primitive types
    const MAX_PRIM_TYPECODE: u8 = 11;
    const TUPLE_TYPECODE: u8 = (TypeCode::MAX_PRIM_TYPECODE + 1) * 8; // 12 * 8 = 96

    const COLLECTION_CONSTR_ID: u8 = 1;

    const OPTION_CONSTR_ID: u8 = 3;

    const TUPLE_PAIR1_CONSTR_ID: u8 = 5;

    const TUPLE_PAIR2_CONSTR_ID: u8 = 6;

    const TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID: u8 = 7;

    /// Parse type code from byte
    fn parse(b: u8) -> Result<Self, SigmaParsingError> {
        match FromPrimitive::from_u8(b) {
            Some(t) => Ok(t),
            None => Err(SigmaParsingError::InvalidTypeCode(b)),
        }
    }

    /// Unpack serialized tag byte. Returns Ok((container, embeddable_type)) if parsing is successful. If embeddable_type is None then need to read-ahead in byte-stream to obtain type
    fn unpack_tag(tag: u8) -> Result<(Option<Self>, Option<Self>), SigmaParsingError> {
        if tag < TypeCode::TUPLE_TYPECODE {
            let container_id =
                (tag / (TypeCode::MAX_PRIM_TYPECODE + 1)) * (TypeCode::MAX_PRIM_TYPECODE + 1);
            let type_id = tag % (TypeCode::MAX_PRIM_TYPECODE + 1);
            let container_code = if container_id == 0 {
                None
            } else {
                Some(TypeCode::parse(container_id)?)
            };
            let type_code = if type_id == 0 {
                None
            } else {
                Some(TypeCode::parse(type_id)?)
            };
            Ok((container_code, type_code))
        } else {
            Ok((None, None))
        }
    }

    fn get_embeddable_type(
        &self,
        tree_version: ErgoTreeVersion,
    ) -> Result<SType, SigmaParsingError> {
        use SType::*;
        // TODO: UnsignedBigInt
        match self {
            TypeCode::SBOOLEAN => Ok(SBoolean),
            TypeCode::SBYTE => Ok(SByte),
            TypeCode::SSHORT => Ok(SShort),
            TypeCode::SINT => Ok(SInt),
            TypeCode::SLONG => Ok(SLong),
            TypeCode::SBIGINT => Ok(SBigInt),
            TypeCode::SGROUP_ELEMENT => Ok(SGroupElement),
            TypeCode::SSIGMAPROP => Ok(SSigmaProp),
            TypeCode::SUNSIGNEDBIGINT if tree_version >= ErgoTreeVersion::V3 => Ok(SUnsignedBigInt),
            _ => Err(SigmaParsingError::InvalidTypeCode(*self as u8)),
        }
    }

    fn from_primitive_type(stype: &SType) -> Option<Self> {
        Some(match stype {
            SType::SAny => TypeCode::SANY,
            SType::SUnit => TypeCode::SUNIT,
            SType::SBoolean => TypeCode::SBOOLEAN,
            SType::SByte => TypeCode::SBYTE,
            SType::SShort => TypeCode::SSHORT,
            SType::SInt => TypeCode::SINT,
            SType::SLong => TypeCode::SLONG,
            SType::SBigInt => TypeCode::SBIGINT,
            SType::SGroupElement => TypeCode::SGROUP_ELEMENT,
            SType::SSigmaProp => TypeCode::SSIGMAPROP,
            SType::SUnsignedBigInt => TypeCode::SUNSIGNEDBIGINT,
            SType::SBox => TypeCode::SBOX,
            SType::SAvlTree => TypeCode::SAVL_TREE,
            SType::SContext => TypeCode::SCONTEXT,
            SType::SString => TypeCode::SSTRING,
            SType::SHeader => TypeCode::SHEADER,
            SType::SPreHeader => TypeCode::SPRE_HEADER,
            SType::SGlobal => TypeCode::SGLOBAL,
            SType::STypeVar(_)
            | SType::SOption(_)
            | SType::SColl(_)
            | SType::STuple(_)
            | SType::SFunc(_) => return None,
        })
    }

    pub(crate) const fn value(&self) -> u8 {
        *self as u8
    }
}

impl SigmaSerializable for TypeCode {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.value())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let b = r.get_u8()?;
        Self::parse(b)
    }
}

impl SType {
    /// Parse type from byte stream. This function should be used instead of
    /// `sigma_parse` when type code is already read for look-ahead
    pub(crate) fn parse_with_tag<R: SigmaByteRead>(
        r: &mut R,
        c: u8,
    ) -> Result<Self, SigmaParsingError> {
        use SType::*;
        if c < TypeCode::TUPLE_TYPECODE {
            let (container, embeddable) = TypeCode::unpack_tag(c)?;
            let mut stype = || {
                embeddable
                    .map(|e| e.get_embeddable_type(r.tree_version()))
                    .unwrap_or_else(|| SType::sigma_parse(r))
            };
            Ok(match container {
                None => {
                    if let Some(embeddable) = embeddable {
                        embeddable.get_embeddable_type(r.tree_version())?
                    } else {
                        return Err(SigmaParsingError::InvalidTypeCode(c));
                    }
                }
                Some(TypeCode::COLL) => SColl(stype()?.into()),
                Some(TypeCode::NESTED_COLL) => SColl(SColl(stype()?.into()).into()),
                Some(TypeCode::OPTION) => SOption(stype()?.into()),
                Some(TypeCode::OPTION_COLL) => SOption(SColl(stype()?.into()).into()),
                Some(TypeCode::TUPLE_PAIR1) => {
                    STuple(stuple::STuple::pair(stype()?, SType::sigma_parse(r)?))
                }
                Some(TypeCode::TUPLE_PAIR2) if embeddable.is_none() => {
                    STuple(stuple::STuple::triple(
                        stype()?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                    ))
                }
                Some(TypeCode::TUPLE_PAIR2) => {
                    let stype = stype()?;
                    STuple(stuple::STuple::pair(SType::sigma_parse(r)?, stype))
                }
                Some(TypeCode::TUPLE_PAIR_SYMMETRIC) if embeddable.is_none() => {
                    STuple(stuple::STuple::quadruple(
                        stype()?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                    ))
                }
                Some(TypeCode::TUPLE_PAIR_SYMMETRIC) => {
                    let stype = stype()?;
                    STuple(stuple::STuple::pair(stype.clone(), stype))
                }
                #[allow(clippy::unreachable)] // All possible container types are checked
                _ => unreachable!(),
            })
        } else {
            Ok(match TypeCode::parse(c)? {
                TypeCode::TUPLE => {
                    let len = r.get_u8()?;
                    let mut items = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        items.push(SType::sigma_parse(r)?);
                    }
                    STuple(stuple::STuple::try_from(items)?)
                }
                TypeCode::SANY => SAny,
                TypeCode::SUNIT => SUnit,
                TypeCode::SBOX => SBox,
                TypeCode::SAVL_TREE => SAvlTree,
                TypeCode::SCONTEXT => SContext,
                TypeCode::SSTRING => SString,
                TypeCode::STYPE_VAR => {
                    STypeVar(crate::types::stype_param::STypeVar::sigma_parse(r)?)
                }
                TypeCode::SHEADER => SHeader,
                TypeCode::SPRE_HEADER => SPreHeader,
                TypeCode::SGLOBAL => SGlobal,
                #[allow(clippy::unreachable)] // All types with typecode >= Tuple are checked
                _ => unreachable!(),
            })
        }
    }
}

/// Each SType is serialized to array of bytes by:
/// - emitting typeCode of each node (see special case for collections below)
/// - then recursively serializing subtrees from left to right on each level
/// - for each collection of primitive type there is special type code to emit single byte instead of two bytes
///
/// Types code intervals
/// - (1 .. MaxPrimTypeCode)  // primitive types
/// - (CollectionTypeCode .. CollectionTypeCode + MaxPrimTypeCode) // collections of primitive types
/// - (MaxCollectionTypeCode ..)  // Other types
///
/// Collection of non-primitive type is serialized as (CollectionTypeCode, serialize(elementType))
impl SigmaSerializable for SType {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        use SType::*;
        match self {
            SType::SFunc(_) => Err(SigmaSerializationError::NotSupported(
                "SFunc serialization is no supported".to_string(),
            )),
            #[allow(clippy::unwrap_used)]
            // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
            stype if stype.is_prim() => TypeCode::from_primitive_type(stype)
                .unwrap()
                .sigma_serialize(w),
            SOption(elem_type) => match &**elem_type {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement | SSigmaProp
                | SUnsignedBigInt => w
                    .put_u8(
                        (TypeCode::OPTION as u8)
                            + (TypeCode::from_primitive_type(elem_type).unwrap() as u8),
                    )
                    .map_err(From::from),
                SColl(inner_elem_type) => match &**inner_elem_type {
                    #[allow(clippy::unwrap_used)]
                    // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                    SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                    | SSigmaProp | SUnsignedBigInt => w
                        .put_u8(
                            (TypeCode::OPTION_COLL as u8)
                                + TypeCode::from_primitive_type(inner_elem_type).unwrap() as u8,
                        )
                        .map_err(From::from),
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                    | SGlobal => {
                        // if not "embeddable" type fallback to generic Option type code following
                        // elem type code
                        TypeCode::OPTION.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SString | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Option type code following
                    // elem type code
                    TypeCode::OPTION.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },

            SType::SColl(elem_type) => match &**elem_type {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement | SSigmaProp
                | SUnsignedBigInt => w
                    .put_u8(
                        TypeCode::COLL as u8
                            + TypeCode::from_primitive_type(elem_type).unwrap() as u8,
                    )
                    .map_err(From::from),
                SColl(inner_elem_type) => match &**inner_elem_type {
                    #[allow(clippy::unwrap_used)]
                    // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                    SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                    | SSigmaProp | SUnsignedBigInt => w
                        .put_u8(
                            TypeCode::NESTED_COLL as u8
                                + TypeCode::from_primitive_type(inner_elem_type).unwrap() as u8,
                        )
                        .map_err(From::from),
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                    | SGlobal => {
                        // if not "embeddable" type fallback to generic Coll type code following
                        // elem type code
                        TypeCode::COLL.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SString | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Coll type code following
                    // elem type code
                    TypeCode::COLL.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },
            SType::STuple(stuple::STuple { items }) => match items.as_slice() {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                [t1, t2] => match (t1, t2) {
                    (
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                        t2,
                    ) if t1 == t2 => w
                        .put_u8(
                            TypeCode::TUPLE_PAIR_SYMMETRIC as u8
                                + TypeCode::from_primitive_type(t1).unwrap() as u8,
                        )
                        .map_err(From::from),
                    (
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                        t2,
                    ) => {
                        w.put_u8(
                            TypeCode::TUPLE_PAIR1 as u8
                                + TypeCode::from_primitive_type(t1).unwrap() as u8,
                        )?;
                        t2.sigma_serialize(w)
                    }
                    (
                        t1,
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                    ) => {
                        w.put_u8(
                            TypeCode::TUPLE_PAIR2 as u8
                                + TypeCode::from_primitive_type(t2).unwrap() as u8,
                        )?;
                        t1.sigma_serialize(w)
                    }
                    (
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                        | SGlobal,
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                        | SGlobal,
                    ) => {
                        // Pair of non-primitive types (`(SBox, SAvlTree)`, `((Int, Byte), (Boolean,Box))`, etc.)
                        TypeCode::TUPLE_PAIR1.sigma_serialize(w)?;
                        t1.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                },
                [t1, t2, t3] => {
                    TypeCode::TUPLE_PAIR2.sigma_serialize(w)?;
                    t1.sigma_serialize(w)?;
                    t2.sigma_serialize(w)?;
                    t3.sigma_serialize(w)
                }
                [t1, t2, t3, t4] => {
                    TypeCode::TUPLE_PAIR_SYMMETRIC.sigma_serialize(w)?;
                    t1.sigma_serialize(w)?;
                    t2.sigma_serialize(w)?;
                    t3.sigma_serialize(w)?;
                    t4.sigma_serialize(w)
                }
                _ => {
                    TypeCode::TUPLE.sigma_serialize(w)?;
                    w.put_u8(items.len() as u8)?;
                    items.iter().try_for_each(|i| i.sigma_serialize(w))
                }
            },

            SType::STypeVar(tv) => {
                TypeCode::STYPE_VAR.sigma_serialize(w)?;
                tv.sigma_serialize(w)
            }
            #[allow(clippy::unreachable)] // Primitive types are covered by if .is_prim() branch
            _ => unreachable!(),
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = r.get_u8()?;
        Self::parse_with_tag(r, c)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<SType>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
