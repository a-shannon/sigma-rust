use std::sync::Arc;

use crate::eval::EvalError;

use ergotree_ir::{
    mir::{
        constant::Constant,
        value::{CollKind, NativeColl, Value},
    },
    serialization::{data::DataSerializer, sigma_byte_writer::SigmaByteWriter},
};

use ergo_chain_types::ec_point::generator;

use super::EvalFn;

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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergo_chain_types::EcPoint;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use ergotree_ir::types::sgroup_elem::GET_ENCODED_METHOD;
    use ergotree_ir::types::stype_param::STypeVar;
    use proptest::proptest;

    use crate::eval::context::Context;
    use crate::eval::tests::{eval_out, eval_out_wo_ctx};
    use ergotree_ir::types::sglobal::{self, SERIALIZE_METHOD};
    use sigma_test_util::force_any_val;

    fn serialize(val: impl Into<Constant>) -> Vec<u8> {
        let constant = val.into();
        let serialize_node = MethodCall::new(
            Expr::Global,
            SERIALIZE_METHOD.clone().with_concrete_types(
                &[(STypeVar::t(), constant.tpe.clone())]
                    .iter()
                    .cloned()
                    .collect(),
            ),
            vec![constant.into()],
        )
        .unwrap();
        eval_out_wo_ctx(&serialize_node.into())
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

    proptest! {
        #[test]
        fn serialize_sigmaprop_eq_prop_bytes(sigma_prop: SigmaProp) {
            let prop_bytes = SigmaPropBytes::try_build(Constant::from(sigma_prop.clone()).into()).unwrap();
            assert_eq!(serialize(sigma_prop), &eval_out_wo_ctx::<Vec<u8>>(&prop_bytes.into())[2..])
        }
    }
}
