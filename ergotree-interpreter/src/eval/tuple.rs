use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Tuple {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let items_v = self
            .items
            .try_mapped_ref(|i| -> Result<Value<'ctx>, EvalError> {
                let v = i.eval(env, ctx)?;
                // Mirror the JVM Tuple eval, which `checkType`s each item
                // (values.scala:801/804): an item whose type is an unsupported tuple
                // (arity != 2) is rejected ("Unsupported tuple type", SType.scala:200).
                // Catches an arity-3 tuple constant carried as a pair item, which the
                // JVM refuses but sigma-rust would otherwise evaluate.
                crate::eval::check_value_type(&i.tpe())?;
                Ok(v)
            });
        Ok(Value::Tup(items_v?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;

    #[test]
    fn eval() {
        let e1: Expr = 1i64.into();
        let e2: Expr = GlobalVars::Height.into();
        let exprs = vec![e1, e2];
        let tuple: Expr = Tuple::new(exprs).unwrap().into();
        let res = eval_out_wo_ctx::<Value>(&tuple);
        assert!(matches!(res, Value::Tup(_)));
    }

    #[test]
    fn eval_rejects_arity3_tuple_item_constant() {
        use crate::eval::test_util::try_eval_out_wo_ctx;
        use ergotree_ir::mir::constant::{Constant, Literal};
        use ergotree_ir::types::stuple::{STuple, TupleItems};
        use ergotree_ir::types::stype::SType;

        // SANTA Tuple.checkType_unsupported (inline): a pair Tuple whose item0 is
        // an arity-3 (Bool,Bool,Bool) tuple CONSTANT. The JVM `checkType`s each
        // Tuple item (values.scala:801/804) → arity != 2 → "Unsupported tuple
        // type" (SType.scala:200-202). sigma-rust models flat N-ary tuples, so the
        // value is representable; it must be rejected at eval to match the JVM.
        let triple = Constant {
            tpe: SType::STuple(STuple::triple(
                SType::SBoolean,
                SType::SBoolean,
                SType::SBoolean,
            )),
            v: Literal::Tup(
                TupleItems::try_from(vec![
                    Literal::Boolean(true),
                    Literal::Boolean(true),
                    Literal::Boolean(true),
                ])
                .unwrap(),
            ),
        };
        let tuple: Expr = Tuple::new(vec![Expr::Const(triple), 1i32.into()])
            .unwrap()
            .into();
        assert!(
            try_eval_out_wo_ctx::<Value>(&tuple).is_err(),
            "arity-3 tuple item constant must be rejected at eval (Unsupported tuple type)"
        );
    }
}
