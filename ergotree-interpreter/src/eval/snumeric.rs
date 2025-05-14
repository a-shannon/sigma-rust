use ergotree_ir::{
    bigint256::BigInt256,
    mir::{constant::TryExtractInto, value::Value},
    types::{
        smethod::SMethod,
        snumeric::{
            self,
            sbigint::{TO_UNSIGNED_METHOD_ID, TO_UNSIGNED_MOD_METHOD_ID},
            sunsignedbigint::{
                MOD_INVERSE_METHOD_ID, MOD_METHOD_ID, MULTIPLY_MOD_METHOD_ID, PLUS_MOD_METHOD_ID,
                SUBTRACT_MOD_METHOD_ID, TO_SIGNED_METHOD_ID,
            },
        },
    },
    unsignedbigint256::UnsignedBigInt,
};
use num_traits::CheckedRem;

use super::{EvalError, EvalFn};

static TO_UNSIGNED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let signed = obj.try_extract_into::<BigInt256>()?;
    UnsignedBigInt::try_from(signed)
        .map_err(|err| EvalError::ArithmeticException(err.into()))
        .map(Value::from)
};

static TO_UNSIGNED_MOD_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let signed = obj.try_extract_into::<BigInt256>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("toUnsignedMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    UnsignedBigInt::from_signed_mod(signed, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("toUnsignedMod: can't divide by 0".into()))
};

static MOD_INVERSE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("modInv: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.mod_inv(modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("modInv: can't divide by 0".into()))
};

static PLUS_MOD_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("plusMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("plusMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_add(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("plusMod: can't divide by 0".into()))
};

static SUBTRACT_MOD_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("subtractMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("subtractMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_sub(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("subtractMod: can't divide by 0".into()))
};

static MULTIPLY_MOD_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("multiplyMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("multiplyMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_mul(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("multiplyMod: can't divide by 0".into()))
};

static MOD_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("mod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_rem(&modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("mod: can't divide by 0".into()))
};

static TO_SIGNED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    BigInt256::try_from(obj)
        .map_err(|e| EvalError::ArithmeticException(e.into()))
        .map(Value::from)
};

// List of methods that are available for all numeric types
fn snumeric_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match method.method_id() {
        // The following methods are explicitly not supported
        snumeric::TO_BYTE_METHOD_ID
        | snumeric::TO_INT_METHOD_ID
        | snumeric::TO_SHORT_METHOD_ID
        | snumeric::TO_LONG_METHOD_ID
        | snumeric::TO_BIGINT_METHOD_ID => Err(EvalError::NotFound("Not implemented".into())),
        _ => Err(EvalError::NotFound(format!(
            "Method {:?} not found",
            method.method_id()
        ))),
    }
}

fn bigint_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match snumeric_evalfn(method) {
        Ok(eval_fn) => Ok(eval_fn),
        Err(_) => match method.method_id() {
            TO_UNSIGNED_METHOD_ID => Ok(TO_UNSIGNED_EVAL_FN),
            TO_UNSIGNED_MOD_METHOD_ID => Ok(TO_UNSIGNED_MOD_EVAL_FN),
            _ => Err(EvalError::NotFound(format!(
                "SBigInt: Method id {:?} not found",
                method.method_id()
            ))),
        },
    }
}

fn unsigned_bigint_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match snumeric_evalfn(method) {
        Ok(eval_fn) => Ok(eval_fn),
        Err(_) => match method.method_id() {
            MOD_INVERSE_METHOD_ID => Ok(MOD_INVERSE_EVAL_FN),
            PLUS_MOD_METHOD_ID => Ok(PLUS_MOD_EVAL_FN),
            SUBTRACT_MOD_METHOD_ID => Ok(SUBTRACT_MOD_EVAL_FN),
            MULTIPLY_MOD_METHOD_ID => Ok(MULTIPLY_MOD_EVAL_FN),
            MOD_METHOD_ID => Ok(MOD_EVAL_FN),
            TO_SIGNED_METHOD_ID => Ok(TO_SIGNED_EVAL_FN),
            _ => Err(EvalError::NotFound(format!(
                "SUnsignedBigInt: Method id {:?} not found",
                method.method_id()
            ))),
        },
    }
}

pub(crate) fn numeric_method_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match method.obj_type.type_code() {
        snumeric::sbyte::TYPE_CODE
        | snumeric::sshort::TYPE_CODE
        | snumeric::sint::TYPE_CODE
        | snumeric::slong::TYPE_CODE => snumeric_evalfn(method),
        snumeric::sbigint::TYPE_CODE => bigint_evalfn(method),
        snumeric::sunsignedbigint::TYPE_CODE => unsigned_bigint_evalfn(method),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Expected numeric type, found {:?}",
            method.obj_type.type_code()
        ))),
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use ergotree_ir::{
        bigint256::BigInt256,
        mir::{constant::Constant, expr::Expr, method_call::MethodCall},
        types::{
            smethod::{SMethod, SMethodDesc},
            snumeric::{
                sbigint::{TO_UNSIGNED_METHOD_DESC, TO_UNSIGNED_MOD_METHOD_DESC},
                sunsignedbigint::{
                    MOD_INVERSE_METHOD_DESC, MOD_METHOD_DESC, MULTIPLY_MOD_METHOD_DESC,
                    PLUS_MOD_METHOD_DESC, SUBTRACT_MOD_METHOD_DESC, TO_SIGNED_METHOD_DESC,
                },
            },
            stype_companion::STypeCompanion,
        },
        unsignedbigint256::UnsignedBigInt,
    };
    use num_traits::CheckedRem;
    use proptest::prelude::*;

    use crate::eval::{test_util::try_eval_out_wo_ctx, EvalError};

    fn eval_modular_op(
        desc: &SMethodDesc,
        args: &[UnsignedBigInt],
    ) -> Result<UnsignedBigInt, EvalError> {
        let mc: Expr = MethodCall::new(
            Constant::from(args[0]).into(),
            SMethod::new(STypeCompanion::SUnsignedBigInt, desc.clone()),
            args[1..]
                .iter()
                .copied()
                .map(Constant::from)
                .map(Expr::from)
                .collect(),
        )
        .unwrap()
        .into();
        try_eval_out_wo_ctx(&mc)
    }

    proptest! {
        #[test]
        fn eval_to_unsigned(signed in any::<BigInt256>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(signed).into(),
                SMethod::new(STypeCompanion::SBigInt, TO_UNSIGNED_METHOD_DESC.clone()),
                vec![],
            )
            .unwrap()
            .into();
            let res = try_eval_out_wo_ctx::<UnsignedBigInt>(&mc);
            if signed < 0.into() {
                assert!(res.is_err())
            } else {
                assert_eq!(res.unwrap().to_string(), signed.to_string());
            }
        }
        #[test]
        fn eval_to_unsigned_mod(signed in any::<BigInt256>(), modulus in any::<UnsignedBigInt>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(signed).into(),
                SMethod::new(STypeCompanion::SBigInt, TO_UNSIGNED_MOD_METHOD_DESC.clone()),
                vec![Constant::from(modulus).into()],
            )
            .unwrap()
            .into();
            assert_eq!(try_eval_out_wo_ctx::<UnsignedBigInt>(&mc).ok(), UnsignedBigInt::from_signed_mod(signed, modulus));
        }

        #[test]
        fn eval_to_signed(unsigned in any::<UnsignedBigInt>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(unsigned).into(),
                SMethod::new(STypeCompanion::SUnsignedBigInt, TO_SIGNED_METHOD_DESC.clone()),
                vec![],
            )
            .unwrap()
            .into();
            let res = try_eval_out_wo_ctx::<BigInt256>(&mc);
            assert_eq!(res.ok(), BigInt256::try_from(unsigned).ok());
        }

        #[test]
        fn eval_mod_ops(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), modulus in any::<UnsignedBigInt>()) {
            assert_eq!(eval_modular_op(&MOD_METHOD_DESC, &[a, modulus]).ok(), a.checked_rem(&modulus));
            assert_eq!(eval_modular_op(&PLUS_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_add(b, modulus));
            assert_eq!(eval_modular_op(&SUBTRACT_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_sub(b, modulus));
            assert_eq!(eval_modular_op(&MULTIPLY_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_mul(b, modulus));
            assert_eq!(eval_modular_op(&MOD_INVERSE_METHOD_DESC, &[a, modulus]).ok(), a.mod_inv(modulus));
        }

    }
}
