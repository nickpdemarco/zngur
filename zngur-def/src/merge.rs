use crate::{
    AdditionalIncludes, ConvertPanicToException, CppRef, CppValue, LayoutPolicy, ZngurExternCppFn,
    ZngurExternCppImpl, ZngurFn, ZngurSpec, ZngurTrait, ZngurType,
};

/// Trait for types with a partial union operation.
///
/// If a type T is Merge, it provides a partial union operation `merge`: T x T -> T.
///
/// Partial unions do not need to be homogenous. If a type U is Merge<T>,
/// it provides a partial union operation `merge`: T X U -> U.
/// For example, T: usize, U: Set<usize>; the partial union is the result of
/// adding the lhs usize to the rhs Set.
///
/// "Partial" means the result is not necessarily defined for all inputs; the union may fail.
/// This is often because the instances are contradictory (as defined by the type).
///
/// There are no guarantees about the state of the mutable argument, `into`, in the case
/// of a failed merge. `merge` is not required to leave `into` in a valid state, or restore
/// it to its original state.
pub trait Merge<T = Self> {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// # Errors
    ///
    /// If the instances are contradictory, a `MergeFailure` is returned.
    fn merge(self, into: &mut T) -> MergeResult;
}

/// The result of a merge operation.
pub type MergeResult = Result<(), MergeFailure>;

/// An unsuccessful merge operation.
pub enum MergeFailure {
    /// The merge was not successful because of a conflict.
    Conflict(String),
}

/// Push an item onto a vector if it is not already present, in linear time.
fn push_unique<T: Eq>(item: T, smallvec: &mut std::vec::Vec<T>) {
    if !smallvec.contains(&item) {
        smallvec.push(item);
    }
}

/// Writes the union of `other` and `smallvec` to the latter in O(N * M) time.
fn inplace_union<T: Eq>(other: Vec<T>, smallvec: &mut std::vec::Vec<T>) {
    for item in other {
        push_unique(item, smallvec);
    }
}

impl<T: Merge> Merge for Option<T> {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// If both `self` and `into` are Some, the underlying values are merged.
    /// Otherwise, the result is whichever value is Some, or None if neither is.
    fn merge(self, into: &mut Self) -> MergeResult {
        match self {
            Some(src) => match into.as_mut() {
                Some(dst) => src.merge(dst),
                None => {
                    *into = Some(src);
                    Ok(())
                }
            },
            None => Ok(()),
        }
    }
}

impl<K, V, I: IntoIterator<Item = (K, V)>> Merge<std::collections::HashMap<K, V>> for I
where
    K: Eq + std::hash::Hash,
    V: Merge,
{
    /// Merges a sequence of key-value pairs into a hash map.
    ///
    /// If a key is present in both `self` and `into`, the corresponding values are merged.
    /// Otherwise, the entry from `self` is inserted into `into`.
    ///
    /// This implementation implies `std::collections::HashMap<K,V>` is `Merge` for all `V: Merge`,
    /// because HashMap is `IntoIterator`. We use `IntoIterator` to allow literal sequences of
    /// key-value pairs to be merged into a map.
    fn merge(self, into: &mut std::collections::HashMap<K, V>) -> MergeResult {
        for (key, value) in self {
            match into.entry(key) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(value);
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    match value.merge(e.get_mut()) {
                        Ok(()) => {}
                        Err(message) => {
                            return Err(message);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Merge for ZngurType {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// PRECONDITION: `self.ty == into.ty`.
    fn merge(self, into: &mut Self) -> MergeResult {
        if self.ty != into.ty {
            panic!(
                "Attempt to merge different types: {} and {}",
                self.ty, into.ty
            );
        }

        if self.layout != into.layout {
            return Err(MergeFailure::Conflict("Layout mismatch".to_string()));
        }

        if self.cpp_ref.is_some() && into.layout != LayoutPolicy::ZERO_SIZED_TYPE {
            return Err(MergeFailure::Conflict(
                "cpp_ref implies a zero sized stack allocated type".to_string(),
            ));
        }

        self.cpp_value.merge(&mut into.cpp_value)?;
        self.cpp_ref.merge(&mut into.cpp_ref)?;

        inplace_union(self.wellknown_traits, &mut into.wellknown_traits);
        inplace_union(self.methods, &mut into.methods);
        inplace_union(self.constructors, &mut into.constructors);
        inplace_union(self.fields, &mut into.fields);

        Ok(())
    }
}

impl Merge for ZngurTrait {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// PRECONDITION: `self.tr == into.tr`.
    fn merge(self, into: &mut Self) -> MergeResult {
        if self.tr != into.tr {
            panic!(
                "Attempt to merge different traits: {} and {}",
                self.tr, into.tr
            );
        }

        inplace_union(self.methods, &mut into.methods);

        Ok(())
    }
}

impl Merge for CppValue {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// There is no meaningful way to merge different CppValues, but we allow
    /// merging the same CppValue from different sources.
    fn merge(self, into: &mut Self) -> MergeResult {
        if self != *into {
            return Err(MergeFailure::Conflict("Cpp value mismatch".to_string()));
        }
        Ok(())
    }
}

impl Merge for CppRef {
    /// Writes the partial union of `self` and `into` to the latter.
    ///
    /// There is no meaningful way to merge different CppRefs, but we allow
    /// merging the same CppRef from different sources.
    fn merge(self, into: &mut Self) -> MergeResult {
        if self != *into {
            return Err(MergeFailure::Conflict("Cpp ref mismatch".to_string()));
        }
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurType {
    /// Merges a type into a specification's type list.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        [(self.ty.clone(), self)].merge(&mut into.types)
    }
}

impl Merge<ZngurSpec> for ZngurTrait {
    /// Merges a trait into a specification's trait list.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        [(self.tr.clone(), self)].merge(&mut into.traits)
    }
}

impl Merge<ZngurSpec> for ZngurFn {
    /// Merges a function into a specification's function list.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.funcs);
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurExternCppFn {
    /// Merges an extern C++ function into a specification's C++ function list.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_funcs);
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurExternCppImpl {
    /// Merges an extern C++ implementation into a specification's C++ implementation list.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_impls);
        Ok(())
    }
}

impl Merge<ZngurSpec> for AdditionalIncludes {
    /// Merges #include directives into a specification's additional includes string.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        into.additional_includes.0 += self.0.as_str();
        Ok(())
    }
}

impl Merge<ZngurSpec> for ConvertPanicToException {
    /// Merges a CPtE flag into a specification's CPtE flag.
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        into.convert_panic_to_exception.0 |= self.0;
        Ok(())
    }
}
