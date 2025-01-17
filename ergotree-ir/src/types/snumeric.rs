use lazy_static::lazy_static;

use crate::{
    ergo_tree::ErgoTreeVersion,
    types::{sfunc::SFunc, stype::SType, stype_param::STypeVar},
};

use super::smethod::{MethodId, SMethodDesc};

/// toByte MethodId
pub const TO_BYTE_METHOD_ID: MethodId = MethodId(1);
/// toShort MethodId
pub const TO_SHORT_METHOD_ID: MethodId = MethodId(2);
/// toInt MethodId
pub const TO_INT_METHOD_ID: MethodId = MethodId(3);
/// toLong MethodId
pub const TO_LONG_METHOD_ID: MethodId = MethodId(4);
/// toBigInt MethodId
pub const TO_BIGINT_METHOD_ID: MethodId = MethodId(5);
/// toBytes MethodId
pub const TO_BYTES_METHOD_ID: MethodId = MethodId(6);
/// toBits MethodId
pub const TO_BITS_METHOD_ID: MethodId = MethodId(7);

// The following methods (toByte, toShort, to...) do not seem to have any implementation in upstream (only methodcall is defined). Since they are used in v6 they are defined here so the methodcalls can be deserialized
lazy_static! {
    static ref TO_BYTE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BYTE_METHOD_ID,
        name: "toByte",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SByte.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_SHORT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_SHORT_METHOD_ID,
        name: "toShort",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SShort.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_INT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_INT_METHOD_ID,
        name: "toInt",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_LONG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_LONG_METHOD_ID,
        name: "toLong",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SLong.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BIGINT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BIGINT_METHOD_ID,
        name: "toBigInt",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SBigInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BYTES_METHOD_ID,
        name: "toBytes",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BITS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BITS_METHOD_ID,
        name: "toBits",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SColl(SType::SBoolean.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    static ref METHOD_DESC: [&'static SMethodDesc; 6] = [
        &TO_BYTE_METHOD_DESC,
        &TO_SHORT_METHOD_DESC,
        &TO_INT_METHOD_DESC,
        &TO_LONG_METHOD_DESC,
        &TO_BYTES_METHOD_DESC,
        &TO_BITS_METHOD_DESC
    ];
}

fn specialize_method(method: &SMethodDesc, tpe: SType) -> SMethodDesc {
    SMethodDesc {
        tpe: method
            .tpe
            .with_subst(&[(STypeVar::t(), tpe)].into_iter().collect()),
        ..method.clone()
    }
}

/// SByte type and methods
pub mod sbyte {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Byte TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SBYTE;
    /// Byte name
    pub const TYPE_NAME: &str = "Byte";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SByte))
            .collect();
    }
}

/// SShort type and methods
pub mod sshort {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SSHORT;
    /// Short Type name
    pub const TYPE_NAME: &str = "Short";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SShort))
            .collect();
    }
}

/// SInt type and methods
pub mod sint {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Int TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SINT;
    /// Int Type name
    pub const TYPE_NAME: &str = "Int";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SInt))
            .collect();
    }
}

/// SLong type and methods
pub mod slong {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SLONG;
    /// Short Type name
    pub const TYPE_NAME: &str = "Long";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SLong))
            .collect();
    }
}

/// SBigInt type and methods
pub mod sbigint {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SBIGINT;
    /// Short Type name
    pub const TYPE_NAME: &str = "BigInt";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SBigInt))
            .collect();
    }
}

/// SUnsignedBigInt type and methods
pub mod sunsignedbigint {
    use super::*;
    use crate::serialization::types::TypeCode;

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SUNSIGNEDBIGINT;
    /// Short Type name
    pub const TYPE_NAME: &str = "UnsignedBigInt";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SUnsignedBigInt))
            .collect();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::{
        bigint256::BigInt256,
        mir::{constant::Constant, method_call::MethodCall},
        serialization::sigma_serialize_roundtrip,
        types::{stype::SType, stype_companion::STypeCompanion},
        unsignedbigint256::UnsignedBigInt,
    };

    #[test]
    fn byte_method_roundtrips() {
        super::sbyte::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SByte))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SByte]);
                let mc = MethodCall::new(Constant::from(1i8).into(), method, vec![]).unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
    #[test]
    fn short_method_roundtrips() {
        super::sshort::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SShort))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SShort]);
                let mc = MethodCall::new(Constant::from(1i16).into(), method, vec![]).unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
    #[test]
    fn int_method_roundtrips() {
        super::sint::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SInt))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SInt]);
                let mc = MethodCall::new(Constant::from(1i32).into(), method, vec![]).unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
    #[test]
    fn long_method_roundtrips() {
        super::slong::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SLong))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SLong]);
                let mc = MethodCall::new(Constant::from(1i64).into(), method, vec![]).unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
    #[test]
    fn bigint_method_roundtrips() {
        super::sbigint::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SBigInt))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SBigInt]);
                let mc = MethodCall::new(Constant::from(BigInt256::from(1)).into(), method, vec![])
                    .unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
    #[test]
    fn unsigned_bigint_method_roundtrips() {
        super::sunsignedbigint::METHOD_DESC
            .iter()
            .map(|m| m.as_method(STypeCompanion::SUnsignedBigInt))
            .for_each(|method| {
                assert_eq!(method.method_raw.tpe.t_dom, [SType::SUnsignedBigInt]);
                let mc = MethodCall::new(
                    Constant::from(UnsignedBigInt::from(1)).into(),
                    method,
                    vec![],
                )
                .unwrap();
                sigma_serialize_roundtrip(&mc);
            });
    }
}
