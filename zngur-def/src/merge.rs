use crate::{
    AdditionalIncludes, ConvertPanicToException, CppRef, CppValue, ZngurExternCppFn,
    ZngurExternCppImpl, ZngurFn, ZngurSpec, ZngurTrait, ZngurType,
};
use std::vec::Vec;

pub trait Merge<T = Self> {
    fn merge(self, into: &mut T) -> MergeResult;
}

pub enum MergeFailure {
    Conflict(String),
}

pub type MergeResult = Result<(), MergeFailure>;

fn push_unique<T: Eq>(item: T, smallvec: &mut Vec<T>) {
    if !smallvec.contains(&item) {
        smallvec.push(item);
    }
}

fn merge_unique<T: Eq>(other: Vec<T>, smallvec: &mut Vec<T>) {
    for item in other {
        push_unique(item, smallvec);
    }
}

impl<T: Merge> Merge for Option<T> {
    fn merge(self, into: &mut Self) -> MergeResult {
        match self {
            Some(a) => match into.as_mut() {
                Some(b) => a.merge(b),
                None => {
                    *into = Some(a);
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

        self.cpp_value.merge(&mut into.cpp_value)?;
        self.cpp_ref.merge(&mut into.cpp_ref)?;

        merge_unique(self.wellknown_traits, &mut into.wellknown_traits);
        merge_unique(self.methods, &mut into.methods);
        merge_unique(self.constructors, &mut into.constructors);
        merge_unique(self.fields, &mut into.fields);

        Ok(())
    }
}

impl Merge for ZngurTrait {
    fn merge(self, into: &mut Self) -> MergeResult {
        if self.tr != into.tr {
            panic!(
                "Attempt to merge different traits: {} and {}",
                self.tr, into.tr
            );
        }

        merge_unique(self.methods, &mut into.methods);

        Ok(())
    }
}

impl Merge for CppValue {
    fn merge(self, into: &mut Self) -> MergeResult {
        if self != *into {
            return Err(MergeFailure::Conflict("Cpp value mismatch".to_string()));
        }
        Ok(())
    }
}

impl Merge for CppRef {
    fn merge(self, into: &mut Self) -> MergeResult {
        if self != *into {
            return Err(MergeFailure::Conflict("Cpp ref mismatch".to_string()));
        }
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurType {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        [(self.ty.clone(), self)].merge(&mut into.types)
    }
}

impl Merge<ZngurSpec> for ZngurTrait {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        [(self.tr.clone(), self)].merge(&mut into.traits)
    }
}

impl Merge<ZngurSpec> for ZngurFn {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.funcs);
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurExternCppFn {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_funcs);
        Ok(())
    }
}

impl Merge<ZngurSpec> for ZngurExternCppImpl {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_impls);
        Ok(())
    }
}

impl Merge<ZngurSpec> for AdditionalIncludes {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        into.additional_includes.0 += self.0.as_str();
        Ok(())
    }
}

impl Merge<ZngurSpec> for ConvertPanicToException {
    fn merge(self, into: &mut ZngurSpec) -> MergeResult {
        // REVISIT: Should we just set to true here?
        into.convert_panic_to_exception.0 |= self.0;
        Ok(())
    }
}
