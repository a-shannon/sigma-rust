use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use crate::types::smethod::SMethod;
use crate::types::stype_companion::STypeCompanion;
use crate::types::stype_param::STypeVar;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// SGlobal type code
pub const TYPE_CODE: TypeCode = TypeCode::SGLOBAL;
/// SGlobal type name
pub static TYPE_NAME: &str = "Global";

/// groupGenerator property
pub const GROUP_GENERATOR_METHOD_ID: MethodId = MethodId(1);
/// "xor" predefined function
pub const XOR_METHOD_ID: MethodId = MethodId(2);
/// serialize function added in v6.0
pub const SERIALIZE_METHOD_ID: MethodId = MethodId(3);
/// "fromBigEndianBytes" predefined function
pub const DESERIALIZE_METHOD_ID: MethodId = MethodId(4);
/// "fromBigEndianBytes" predefined function
pub const FROM_BIGENDIAN_BYTES_METHOD_ID: MethodId = MethodId(5);
/// "some" property
pub const SOME_METHOD_ID: MethodId = MethodId(9);
/// "none" property
pub const NONE_METHOD_ID: MethodId = MethodId(10);

lazy_static! {
    /// Global method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![GROUP_GENERATOR_METHOD_DESC.clone(), XOR_METHOD_DESC.clone(), SERIALIZE_METHOD_DESC.clone(), DESERIALIZE_METHOD_DESC.clone(), FROM_BIGENDIAN_BYTES_METHOD_DESC.clone(), NONE_METHOD_DESC.clone(), SOME_METHOD_DESC.clone()];
}

lazy_static! {
    static ref GROUP_GENERATOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GROUP_GENERATOR_METHOD_ID,
        name: "groupGenerator",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal],
            t_range: SType::SGroupElement.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
     /// GLOBAL.GroupGenerator
    pub static ref GROUP_GENERATOR_METHOD: SMethod = SMethod::new(STypeCompanion::Global, GROUP_GENERATOR_METHOD_DESC.clone(),);

}

lazy_static! {
    static ref XOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: XOR_METHOD_ID,
        name: "xor",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SColl(SType::SByte.into()),
                SType::SColl(SType::SByte.into()),
            ],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
     /// GLOBAL.xor
    pub static ref XOR_METHOD: SMethod = SMethod::new(STypeCompanion::Global, XOR_METHOD_DESC.clone(),);

}

lazy_static! {
    static ref FROM_BIGENDIAN_BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: FROM_BIGENDIAN_BYTES_METHOD_ID,
        name: "fromBigEndianBytes",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal, SType::SColl(SType::SByte.into())],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.fromBigEndianBytes
    pub static ref FROM_BIGENDIAN_BYTES_METHOD: SMethod = SMethod::new(STypeCompanion::Global, FROM_BIGENDIAN_BYTES_METHOD_DESC.clone(),);

    static ref DESERIALIZE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DESERIALIZE_METHOD_ID,
        name: "deserialize",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SColl(SType::SByte.into())
            ],
            t_range: Box::new(STypeVar::t().into()),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.deserialize
    pub static ref DESERIALIZE_METHOD: SMethod = SMethod::new(STypeCompanion::Global, DESERIALIZE_METHOD_DESC.clone(),);

    static ref SERIALIZE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SERIALIZE_METHOD_ID,
        name: "serialize",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                STypeVar::t().into()
            ],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.serialize
    pub static ref SERIALIZE_METHOD: SMethod = SMethod::new(STypeCompanion::Global, SERIALIZE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref SOME_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SOME_METHOD_ID,
        name: "some",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal, SType::STypeVar(STypeVar::t())],
            t_range:SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.some
    pub static ref SOME_METHOD : SMethod = SMethod::new(STypeCompanion::Global, SOME_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref NONE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: NONE_METHOD_ID,
        name: "none",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal],
            t_range:SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.none
    pub static ref NONE_METHOD : SMethod = SMethod::new(STypeCompanion::Global, NONE_METHOD_DESC.clone(),);
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use proptest::prelude::*;

    use crate::{
        mir::{expr::Expr, method_call::MethodCall},
        serialization::SigmaSerializable,
        types::{stype::SType, stype_param::STypeVar},
    };

    use super::DESERIALIZE_METHOD;
    proptest! {
       #[test]
       fn test_deserialize_method_roundtrip(v in any::<SType>()) {
           let type_args = core::iter::once((STypeVar::t(), v)).collect();
           let mc = MethodCall::with_type_args(
               Expr::Global,
               DESERIALIZE_METHOD.clone().with_concrete_types(&type_args),
               vec![vec![0i8].into()],
               type_args,
           ).unwrap();
           assert_eq!(MethodCall::sigma_parse_bytes(&mc.sigma_serialize_bytes().unwrap()).unwrap(), mc);
       }
    }
}
