use crate::{
    AdditionalIncludes, ConvertPanicToException, ZngurExternCppFn, ZngurExternCppImpl, ZngurFile,
    ZngurFn, ZngurTrait, ZngurType,
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

        merge_unique(self.wellknown_traits, &mut into.wellknown_traits);
        merge_unique(self.methods, &mut into.methods);
        merge_unique(self.constructors, &mut into.constructors);
        // TODO: cpp_value, cpp_ref

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

impl Merge<ZngurFile> for ZngurType {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        [(self.ty.clone(), self)].merge(&mut into.types)
    }
}

impl Merge<ZngurFile> for ZngurTrait {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        [(self.tr.clone(), self)].merge(&mut into.traits)
    }
}

impl Merge<ZngurFile> for ZngurFn {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        push_unique(self, &mut into.funcs);
        Ok(())
    }
}

impl Merge<ZngurFile> for ZngurExternCppFn {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_funcs);
        Ok(())
    }
}

impl Merge<ZngurFile> for ZngurExternCppImpl {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        push_unique(self, &mut into.extern_cpp_impls);
        Ok(())
    }
}

impl Merge<ZngurFile> for AdditionalIncludes {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        into.additional_includes.0 += self.0.as_str();
        Ok(())
    }
}

impl Merge<ZngurFile> for ConvertPanicToException {
    fn merge(self, into: &mut ZngurFile) -> MergeResult {
        // REVISIT: Should we just set to true here?
        into.convert_panic_to_exception.0 |= self.0;
        Ok(())
    }
}
