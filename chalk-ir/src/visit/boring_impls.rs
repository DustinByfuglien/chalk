//! This module contains "rote and uninteresting" impls of `Visit` for
//! various types. In general, we prefer to derive `Visit`, but
//! sometimes that doesn't work for whatever reason.
//!
//! The more interesting impls of `Visit` remain in the `visit` module.

use crate::{
    AdtId, AssocTypeId, ClausePriority, ClosureId, DebruijnIndex, FloatTy, FnDefId, GenericArg,
    Goals, ImplId, IntTy, Interner, Mutability, OpaqueTyId, PlaceholderIndex, ProgramClause,
    ProgramClauses, QuantifiedWhereClauses, QuantifierKind, Scalar, Substitution, SuperVisit,
    TraitId, UintTy, UniverseIndex, Visit, VisitResult, Visitor,
};
use std::{marker::PhantomData, sync::Arc};

/// Convenience function to visit all the items in the iterator it.
pub fn visit_iter<'i, T, I, R>(
    it: impl Iterator<Item = T>,
    visitor: &mut dyn Visitor<'i, I, Result = R>,
    outer_binder: DebruijnIndex,
) -> R
where
    T: Visit<I>,
    I: 'i + Interner,
    R: VisitResult,
{
    let mut result = R::new();
    for e in it {
        result = result.combine(e.visit_with(visitor, outer_binder));
        if result.return_early() {
            return result;
        }
    }
    result
}

impl<T: Visit<I>, I: Interner> Visit<I> for &T {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Vec<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for &[T] {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        visit_iter(self.iter(), visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Box<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

impl<T: Visit<I>, I: Interner> Visit<I> for Arc<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        T::visit_with(self, visitor, outer_binder)
    }
}

macro_rules! tuple_visit {
    ($($n:ident),*) => {
        impl<$($n: Visit<I>,)* I: Interner> Visit<I> for ($($n,)*) {
            fn visit_with<'i, R: VisitResult>(&self, visitor: &mut dyn Visitor<'i, I, Result = R>, outer_binder: DebruijnIndex) -> R where I: 'i
            {
                #[allow(non_snake_case)]
                let &($(ref $n),*) = self;
                let mut result = R::new();
                $(
                    result = result.combine($n.visit_with(visitor, outer_binder));
                    if result.return_early() { return result; }
                )*
                result
            }
        }
    }
}

tuple_visit!(A, B);
tuple_visit!(A, B, C);
tuple_visit!(A, B, C, D);
tuple_visit!(A, B, C, D, E);

impl<T: Visit<I>, I: Interner> Visit<I> for Option<T> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        match self {
            Some(e) => e.visit_with(visitor, outer_binder),
            None => R::new(),
        }
    }
}

impl<I: Interner> Visit<I> for GenericArg<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        self.data(interner).visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for Substitution<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for Goals<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();
        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

#[macro_export]
macro_rules! const_visit {
    ($t:ty) => {
        impl<I: Interner> $crate::visit::Visit<I> for $t {
            fn visit_with<'i, R: VisitResult>(
                &self,
                _visitor: &mut dyn ($crate::visit::Visitor<'i, I, Result = R>),
                _outer_binder: DebruijnIndex,
            ) -> R
            where
                I: 'i,
            {
                R::new()
            }
        }
    };
}

const_visit!(bool);
const_visit!(usize);
const_visit!(UniverseIndex);
const_visit!(PlaceholderIndex);
const_visit!(QuantifierKind);
const_visit!(DebruijnIndex);
const_visit!(ClausePriority);
const_visit!(());
const_visit!(Scalar);
const_visit!(UintTy);
const_visit!(IntTy);
const_visit!(FloatTy);
const_visit!(Mutability);

#[macro_export]
macro_rules! id_visit {
    ($t:ident) => {
        impl<I: Interner> $crate::visit::Visit<I> for $t<I> {
            fn visit_with<'i, R: VisitResult>(
                &self,
                _visitor: &mut dyn ($crate::visit::Visitor<'i, I, Result = R>),
                _outer_binder: DebruijnIndex,
            ) -> R
            where
                I: 'i,
            {
                R::new()
            }
        }
    };
}

id_visit!(ImplId);
id_visit!(AdtId);
id_visit!(TraitId);
id_visit!(OpaqueTyId);
id_visit!(AssocTypeId);
id_visit!(FnDefId);
id_visit!(ClosureId);

impl<I: Interner> SuperVisit<I> for ProgramClause<I> {
    fn super_visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        self.data(interner).0.visit_with(visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for ProgramClauses<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for QuantifiedWhereClauses<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        visitor: &mut dyn Visitor<'i, I, Result = R>,
        outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        let interner = visitor.interner();

        visit_iter(self.iter(interner), visitor, outer_binder)
    }
}

impl<I: Interner> Visit<I> for PhantomData<I> {
    fn visit_with<'i, R: VisitResult>(
        &self,
        _visitor: &mut dyn Visitor<'i, I, Result = R>,
        _outer_binder: DebruijnIndex,
    ) -> R
    where
        I: 'i,
    {
        R::new()
    }
}
