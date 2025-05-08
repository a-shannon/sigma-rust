//! Traversable trait

use crate::mir::{expr::Expr, unary_op::OneArgOp};
use alloc::boxed::Box;

/// Trait for types that have child nodes.
/// In ergotree-ir this is used for traversing trees of [`Expr`] and doing rewriting operations such as replacing [ConstantPlaceholder](crate::mir::constant::ConstantPlaceholder)s with [`Constant`](crate::mir::constant::Constant)s
pub trait Traversable {
    /// Type that implementor has edges to. Since [Self::Item] also implements Traversable this can be used to traverse the entire tree
    type Item: Traversable;
    /// Return an iterator for child nodes
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Item> + 'a>;
    /// Return a iterator for mutable access to all child nodes
    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut Self::Item> + 'a>;
}

impl<T: OneArgOp> Traversable for T {
    type Item = Expr;

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Item> + 'a> {
        Box::new(core::iter::once(self.input()))
    }
    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut Self::Item> + 'a> {
        Box::new(core::iter::once(self.input_mut()))
    }
}

macro_rules! iter_from {
    ($self:ident, opt $x:tt, $($y:tt)*) => {
        $self.$x.as_mut().into_iter().chain(crate::traversable::iter_from!($self, $($y)*))
    };
    ($self:ident, arr $x:tt, $($y:tt)*) => {
        $self.$x.iter().chain(crate::traversable::iter_from!($self, $($y)*))
    };
    ($self:ident, boxed $x:tt, $($y:tt)*) => {
        core::iter::once(&*$self.$x).chain(crate::traversable::iter_from!($self, $($y)*))
    };
    ($self:ident, opt $x:tt) => {
        $self.$x.as_deref().into_iter()
    };
    ($self:ident, arr $x:tt) => {
        $self.$x.iter()
    };
    ($self:ident, boxed $x:tt) => {
        core::iter::once(&*$self.$x)
    };
    ($self:ident) => {
        core::iter::empty()
    }
}

macro_rules! iter_from_mut {
    ($self:ident, opt $x:tt, $($y:tt)*) => {
        $self.$x.as_mut().into_iter().chain(crate::traversable::iter_from_mut!($self, $($y)*))
    };
    ($self:ident, arr $x:tt, $($y:tt)*) => {
        $self.$x.iter_mut().chain(crate::traversable::iter_from_mut!($self, $($y)*))
    };
    ($self:ident, boxed $x:tt, $($y:tt)*) => {
        core::iter::once(&mut *$self.$x).chain(crate::traversable::iter_from_mut!($self, $($y)*))
    };
    ($self:ident, opt $x:tt) => {
        $self.$x.as_deref_mut().into_iter()
    };
    ($self:ident, arr $x:tt) => {
        $self.$x.iter_mut()
    };
    ($self:ident, boxed $x:tt) => {
        core::iter::once(&mut *$self.$x)
    };
    ($self:ident) => {
        core::iter::empty()
    }
}

// Implement Traversable<Item = Expr> for an Evaluable Node.
macro_rules! impl_traversable_expr {
    ($op:ident $(, $($args:tt)+ )? ) => {
        impl crate::traversable::Traversable for $op {
            type Item = Expr;
            fn children(&self) -> alloc::boxed::Box<dyn Iterator<Item = &Self::Item> + '_> {
                alloc::boxed::Box::new(crate::traversable::iter_from!(self $(, $($args)*)?))
            }
            fn children_mut(&mut self) -> alloc::boxed::Box<dyn Iterator<Item = &mut Self::Item> + '_> {
                alloc::boxed::Box::new(crate::traversable::iter_from_mut!(self $(, $($args)*)?))
            }
        }
    };
}

pub(crate) use impl_traversable_expr;
pub(crate) use iter_from;
pub(crate) use iter_from_mut;

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    use crate::mir::{constant::Constant, expr::Expr};

    use super::Traversable;

    struct TestStruct {
        a: Box<Expr>,
        b: Box<Expr>,
        c: Vec<Expr>,
        d: Option<Box<Expr>>,
    }
    impl_traversable_expr!(TestStruct, boxed a, boxed b, arr c, opt d);
    struct EmptyStruct;

    impl_traversable_expr!(EmptyStruct);
    #[test]
    fn test_impl_traversable() {
        let mut test_struct = TestStruct {
            a: Box::new(Expr::from(Constant::from(1i8))),
            b: Box::new(Expr::from(Constant::from(2i8))),
            c: vec![Expr::from(Constant::from(3i8))],
            d: Some(Box::new(Expr::from(Constant::from(4i8)))),
        };
        test_struct
            .children_mut()
            .zip(1..=4i8)
            .for_each(|(c, i)| assert_eq!(&*c, &Constant::from(i).into()));
        let mut empty = EmptyStruct;
        assert_eq!(empty.children_mut().count(), 0);
    }
}
