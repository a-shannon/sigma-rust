use crate::serialization::types::TypeCode;
use crate::types::stype_param::STypeVar;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use crate::types::smethod::SMethod;
use crate::types::stype_companion::STypeCompanion;
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

lazy_static! {
    /// Global method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![&GROUP_GENERATOR_METHOD_DESC, &XOR_METHOD_DESC, &SERIALIZE_METHOD_DESC]; // TODO: versioning
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
        explicit_type_args: vec![]
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
        explicit_type_args: vec![]
    };
     /// GLOBAL.xor
    pub static ref XOR_METHOD: SMethod = SMethod::new(STypeCompanion::Global, XOR_METHOD_DESC.clone(),);

}

lazy_static! {
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
        explicit_type_args: vec![]
    };
     /// GLOBAL.serialize
    pub static ref SERIALIZE_METHOD: SMethod = SMethod::new(STypeCompanion::Global, SERIALIZE_METHOD_DESC.clone(),);

}
