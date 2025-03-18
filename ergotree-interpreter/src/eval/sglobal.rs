use crate::eval::EvalError;
use alloc::boxed::Box;
use alloc::{string::ToString, sync::Arc};

use ergotree_ir::{
    mir::{
        constant::{Constant, TryExtractInto},
        value::{CollKind, NativeColl, Value},
    },
    serialization::{
        data::DataSerializer,
        sigma_byte_reader::{self, SigmaByteRead},
        sigma_byte_writer::SigmaByteWriter,
    },
};

use super::EvalFn;
use crate::eval::Vec;
use ergo_chain_types::ec_point::generator;
use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::types::stype::SType;

fn helper_xor(x: &[i8], y: &[i8]) -> Arc<[i8]> {
    x.iter().zip(y.iter()).map(|(x1, x2)| *x1 ^ *x2).collect()
}

pub(crate) static GROUP_GENERATOR_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::from(generator()))
};

pub(crate) static XOR_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.xor expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let right_v = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("xor: missing right arg".to_string()))?;
    let left_v = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("xor: missing left arg".to_string()))?;

    match (left_v.clone(), right_v.clone()) {
        (
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(l_byte))),
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(r_byte))),
        ) => {
            let xor = helper_xor(&l_byte, &r_byte);
            Ok(CollKind::NativeColl(NativeColl::CollByte(xor)).into())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected Xor input to be byte array, got: {0:?}",
            (left_v, right_v)
        ))),
    }
};

pub(crate) static SGLOBAL_FROM_BIGENDIAN_BYTES_EVAL_FN: EvalFn = |mc, _env, _ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.fromBigEndianBytes expected obj to be Value::Global, got {:?}",
            obj
        )));
    }

    let bytes_val = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("fromBigEndianBytes: missing bytes arg".to_string()))?;
    let type_val = mc.tpe().t_range.clone();

    let bytes = match bytes_val {
        Value::Coll(CollKind::NativeColl(NativeColl::CollByte(bytes))) => bytes,
        _ => {
            return Err(EvalError::UnexpectedValue(format!(
                "fromBigEndianBytes: expected first argument to be byte array, got {:?}",
                bytes_val
            )))
        }
    };

    match *type_val {
        SType::SByte => {
            if bytes.len() != 1 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Byte with fromBigEndianBytes, exactly one byte should be provided".to_string(),
                ));
            }
            Ok(Value::Byte(bytes[0]))
        }
        SType::SShort => {
            if bytes.len() != 2 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Short with fromBigEndianBytes, exactly two bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i16, |acc, &x| (acc << 8) | (x as u8 as i16));
            Ok(Value::Short(value))
        }
        SType::SInt => {
            if bytes.len() != 4 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Int with fromBigEndianBytes, exactly four bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i32, |acc, &x| (acc << 8) | (x as u8 as i32));
            Ok(Value::Int(value))
        }
        SType::SLong => {
            if bytes.len() != 8 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Long with fromBigEndianBytes, exactly eight bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i64, |acc, &x| (acc << 8) | (x as u8 as i64));
            Ok(Value::Long(value))
        }
        SType::SBigInt => {
            if bytes.len() > 32 {
                return Err(EvalError::UnexpectedValue(
                    "BigInt value doesn't fit into 32 bytes in fromBigEndianBytes".to_string(),
                ));
            }
            let bytes_vec: Vec<u8> = bytes.iter().map(|&x| x as u8).collect();
            Ok(Value::BigInt(
                BigInt256::from_be_slice(&bytes_vec).ok_or_else(|| {
                    EvalError::UnexpectedValue("Failed to convert to BigInt256".to_string())
                })?,
            ))
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Unsupported type provided in fromBigEndianBytes: {:?}",
            type_val
        ))),
    }
};

pub(crate) static DESERIALIZE_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.deserialize expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let output_type = &mc.tpe().t_range;
    let bytes = args
        .first()
        .ok_or_else(|| EvalError::NotFound("deserialize: missing first arg".into()))?
        .clone()
        .try_extract_into::<Vec<u8>>()?;
    let mut reader = sigma_byte_reader::from_bytes(&bytes);
    Ok(Value::from(
        reader.with_tree_version(ctx.tree_version(), |reader| {
            DataSerializer::sigma_parse(output_type, reader)
        })?,
    ))
};

pub(crate) static SERIALIZE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let arg: Constant = args
        .first()
        .ok_or_else(|| EvalError::NotFound("serialize: missing first arg".into()))?
        .to_static()
        .try_into()
        .map_err(EvalError::UnexpectedValue)?;

    let mut buf = vec![];
    let mut writer = SigmaByteWriter::new(&mut buf, None);
    DataSerializer::sigma_serialize(&arg.v, &mut writer)?;
    Ok(Value::from(buf))
};

pub(crate) static SGLOBAL_SOME_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.some expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let value = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("some: missing value arg".to_string()))?;
    Ok(Value::Opt(Some(Box::new(value))))
};

pub(crate) static SGLOBAL_NONE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.none expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::Opt(None))
};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergo_chain_types::EcPoint;
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use ergotree_ir::types::sgroup_elem::GET_ENCODED_METHOD;
    use ergotree_ir::types::stype_param::STypeVar;
    use proptest::proptest;

    use crate::eval::tests::{eval_out, eval_out_wo_ctx, try_eval_out_with_version};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::types::sglobal::{self, DESERIALIZE_METHOD, SERIALIZE_METHOD};
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    fn serialize(val: impl Into<Constant>) -> Vec<u8> {
        let constant = val.into();
        let serialize_node = MethodCall::new(
            Expr::Global,
            SERIALIZE_METHOD.clone().with_concrete_types(
                &[(STypeVar::t(), constant.tpe.clone())]
                    .into_iter()
                    .collect(),
            ),
            vec![constant.into()],
        )
        .unwrap();
        let ctx = force_any_val::<Context>();
        assert!((0u8..ErgoTreeVersion::V3.into()).all(|version| {
            try_eval_out_with_version::<Vec<u8>>(&serialize_node.clone().into(), &ctx, version, 3)
                .is_err()
        }));
        try_eval_out_with_version(&serialize_node.into(), &ctx, ErgoTreeVersion::V3.into(), 3)
            .unwrap()
    }
    fn deserialize(array: &[u8], return_type: SType) -> Constant {
        let type_args = [(STypeVar::t(), return_type)].into_iter().collect();
        let deserialize_node = MethodCall::with_type_args(
            Expr::Global,
            DESERIALIZE_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(array.to_owned()).into()],
            type_args,
        )
        .unwrap();
        let ctx = force_any_val::<Context>();
        assert!((0u8..ErgoTreeVersion::V3.into()).all(|version| {
            try_eval_out_with_version::<Vec<u8>>(&deserialize_node.clone().into(), &ctx, version, 3)
                .is_err()
        }));
        try_eval_out_with_version::<Value>(
            &deserialize_node.into(),
            &ctx,
            ErgoTreeVersion::V3.into(),
            3,
        )
        .unwrap()
        .try_into()
        .unwrap()
    }

    fn create_some_none_method_call<T>(value: Option<T>, tpe: SType) -> Expr
    where
        T: Into<Constant>,
    {
        let type_args = std::iter::once((STypeVar::t(), tpe.clone())).collect();
        match value {
            Some(v) => MethodCall::new(
                Expr::Global,
                sglobal::SOME_METHOD.clone().with_concrete_types(&type_args),
                vec![Expr::Const(v.into())],
            )
            .unwrap()
            .into(),
            None => MethodCall::with_type_args(
                Expr::Global,
                sglobal::NONE_METHOD.clone().with_concrete_types(&type_args),
                vec![],
                type_args,
            )
            .unwrap()
            .into(),
        }
    }

    #[test]
    fn eval_group_generator() {
        let expr: Expr = PropertyCall::new(Expr::Global, sglobal::GROUP_GENERATOR_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<EcPoint>(&expr, &ctx),
            ergo_chain_types::ec_point::generator()
        );
    }

    #[test]
    fn eval_xor() {
        let left = vec![1_i8, 1, 0, 0];
        let right = vec![0_i8, 1, 0, 1];
        let expected_xor = vec![1_i8, 0, 0, 1];

        let expr: Expr = MethodCall::new(
            Expr::Global,
            sglobal::XOR_METHOD.clone(),
            vec![right.into(), left.into()],
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<i8>>(&expr, &ctx).as_slice(),
            expected_xor.as_slice()
        );
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn test_bigendian_bytes_roundtrip(
            v_byte in any::<i8>(),
            v_short in any::<i16>(),
            v_int in any::<i32>(),
            v_long in any::<i64>()
        ) {
            {
                let bytes = vec![v_byte];

                let type_args = std::iter::once((STypeVar::t(), SType::SByte)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i8>(&expr), v_byte);
            }

            {
                let bytes = vec![(v_short >> 8) as i8, v_short as i8];

                let type_args = std::iter::once((STypeVar::t(), SType::SShort)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i16>(&expr), v_short);
            }

            {
                let bytes = vec![
                    (v_int >> 24) as i8,
                    (v_int >> 16) as i8,
                    (v_int >> 8) as i8,
                    v_int as i8
                ];

                let type_args = std::iter::once((STypeVar::t(), SType::SInt)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i32>(&expr), v_int);
            }

            {
                let bytes = vec![
                    (v_long >> 56) as i8,
                    (v_long >> 48) as i8,
                    (v_long >> 40) as i8,
                    (v_long >> 32) as i8,
                    (v_long >> 24) as i8,
                    (v_long >> 16) as i8,
                    (v_long >> 8) as i8,
                    v_long as i8
                ];

                let type_args = std::iter::once((STypeVar::t(), SType::SLong)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.clone().into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i64>(&expr), v_long);

                let original_long = ((bytes[0] as i64) << 56) |
                                  (((bytes[1] as i64) & 0xFF) << 48) |
                                  (((bytes[2] as i64) & 0xFF) << 40) |
                                  (((bytes[3] as i64) & 0xFF) << 32) |
                                  (((bytes[4] as i64) & 0xFF) << 24) |
                                  (((bytes[5] as i64) & 0xFF) << 16) |
                                  (((bytes[6] as i64) & 0xFF) << 8) |
                                  ((bytes[7] as i64) & 0xFF);
                assert_eq!(original_long, v_long);
            }
        }

        #[test]
        fn test_bigint_roundtrip(v_long in any::<i64>()) {
            let bytes = vec![
                (v_long >> 56) as i8,
                (v_long >> 48) as i8,
                (v_long >> 40) as i8,
                (v_long >> 32) as i8,
                (v_long >> 24) as i8,
                (v_long >> 16) as i8,
                (v_long >> 8) as i8,
                v_long as i8
            ];

            let type_args = std::iter::once((STypeVar::t(), SType::SBigInt)).collect();
            let expr: Expr = MethodCall::with_type_args(
                Expr::Global,
                sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                vec![bytes.into()],
                type_args,
            )
            .unwrap()
            .into();
            assert_eq!(eval_out_wo_ctx::<BigInt256>(&expr), BigInt256::from(v_long));
        }

        #[test]
        fn test_some_and_none(
            byte_val in any::<i8>(),
            int_val in any::<i32>(),
            long_val in any::<i64>()
        ) {
            assert_eq!(eval_out_wo_ctx::<Option<i8>>(&create_some_none_method_call(Some(byte_val), SType::SByte)), Some(byte_val));
            assert_eq!(eval_out_wo_ctx::<Option<i32>>(&create_some_none_method_call(Some(int_val), SType::SInt)), Some(int_val));
            assert_eq!(eval_out_wo_ctx::<Option<i64>>(&create_some_none_method_call(Some(long_val), SType::SLong)), Some(long_val));
            assert_eq!(eval_out_wo_ctx::<Option<i8>>(&create_some_none_method_call::<i8>(None, SType::SByte)), None);
            assert_eq!(eval_out_wo_ctx::<Option<i64>>(&create_some_none_method_call::<i64>(None, SType::SLong)), None);
        }

    }

    #[test]
    fn serialize_byte() {
        assert_eq!(serialize(-128i8), vec![-128i8 as u8]);
        assert_eq!(serialize(-1i8), vec![-1i8 as u8]);
        assert_eq!(serialize(0i8), vec![0u8]);
        assert_eq!(serialize(1i8), vec![1]);
        assert_eq!(serialize(127i8), vec![127u8]);
    }

    #[test]
    fn serialize_short() {
        assert_eq!(serialize(i16::MIN), vec![0xff, 0xff, 0x03]);
        assert_eq!(serialize(-1i16), vec![0x01]);
        assert_eq!(serialize(0i16), vec![0x00]);
        assert_eq!(serialize(1i16), vec![0x02]);
        assert_eq!(serialize(i16::MAX), vec![0xfe, 0xff, 0x03]);
    }

    #[test]
    fn serialize_byte_array() {
        let arr = vec![0xc0, 0xff, 0xee];
        let serialized = serialize(arr.clone());

        assert_eq!(serialized[0], arr.len() as u8);
        assert_eq!(&serialized[1..], &arr)
    }

    // test that serialize(long) != longToByteArray()
    #[test]
    fn serialize_long_ne_tobytearray() {
        let num = -1000i64;
        let long_to_byte_array = LongToByteArray::try_build(Constant::from(num).into()).unwrap();
        let serialized = serialize(num);
        assert!(serialized != eval_out_wo_ctx::<Vec<u8>>(&long_to_byte_array.into()))
    }

    // test equivalence between Global.serialize and ge.getEncoded
    #[test]
    fn serialize_group_element() {
        let ec_point = EcPoint::from_base16_str(String::from(
            "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
        ))
        .unwrap();
        let get_encoded = MethodCall::new(
            Constant::from(ec_point.clone()).into(),
            GET_ENCODED_METHOD.clone(),
            vec![],
        )
        .unwrap();
        assert_eq!(
            eval_out_wo_ctx::<Vec<u8>>(&get_encoded.into()),
            serialize(ec_point)
        );
    }

    #[test]
    fn deserialize_group_element() {
        let ec_point = EcPoint::from_base16_str(String::from(
            "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
        ))
        .unwrap();
        let get_encoded = MethodCall::new(
            Constant::from(ec_point.clone()).into(),
            GET_ENCODED_METHOD.clone(),
            vec![],
        )
        .unwrap();
        let encoded = eval_out_wo_ctx::<Vec<u8>>(&get_encoded.into());
        assert_eq!(
            deserialize(&encoded, SType::SGroupElement),
            Constant::from(ec_point)
        );
    }

    proptest! {
        #[test]
        fn serialize_sigmaprop_eq_prop_bytes(sigma_prop: SigmaProp) {
            let prop_bytes_op = SigmaPropBytes::try_build(Constant::from(sigma_prop.clone()).into()).unwrap();
            let prop_bytes = eval_out_wo_ctx::<Vec<u8>>(&prop_bytes_op.into());
            assert_eq!(serialize(sigma_prop.clone()), &prop_bytes[2..]);
            assert_eq!(deserialize(&prop_bytes[2..], SType::SSigmaProp), sigma_prop.into());
        }
        #[test]
        fn serialize_roundtrip(v in any::<Constant>()) {
            let tpe = v.tpe.clone();
            let res = std::panic::catch_unwind(|| assert_eq!(deserialize(&serialize(v.clone()), tpe.clone()), v));
            if matches!(tpe, SType::SOption(_)) {
                assert!(res.is_err());
            }
            else {
                res.unwrap();
            }
        }
    }
}
