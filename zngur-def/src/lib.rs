use std::{collections::HashMap, fmt::Display};

use cargo_metadata::{DependencyKind, MetadataCommand};
use itertools::Itertools;

mod merge;
pub use merge::{Merge, MergeFailure, MergeResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Mut,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZngurMethodReceiver {
    Static,
    Ref(Mutability),
    Move,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurMethod {
    pub name: String,
    pub generics: Vec<RustType>,
    pub receiver: ZngurMethodReceiver,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurFn {
    pub path: RustPathAndGenerics,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurExternCppFn {
    pub name: String,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurExternCppImpl {
    pub tr: Option<RustTrait>,
    pub ty: RustType,
    pub methods: Vec<ZngurMethod>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurConstructor {
    pub name: Option<String>,
    pub inputs: Vec<(String, RustType)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurField {
    pub name: String,
    pub ty: RustType,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZngurWellknownTrait {
    Debug,
    Drop,
    Unsized,
    Copy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ZngurWellknownTraitData {
    Debug {
        pretty_print: String,
        debug_print: String,
    },
    Drop {
        drop_in_place: String,
    },
    Unsized,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPolicy {
    StackAllocated { size: usize, align: usize },
    HeapAllocated,
    OnlyByRef,
}

impl LayoutPolicy {
    pub const ZERO_SIZED_TYPE: Self = LayoutPolicy::StackAllocated { size: 0, align: 1 };
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurMethodDetails {
    pub data: ZngurMethod,
    pub use_path: Option<Vec<String>>,
    pub deref: Option<RustType>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CppValue(pub String, pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct CppRef(pub String);

impl Display for CppRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ZngurType {
    pub ty: RustType,
    pub layout: LayoutPolicy,
    pub wellknown_traits: Vec<ZngurWellknownTrait>,
    pub methods: Vec<ZngurMethodDetails>,
    pub constructors: Vec<ZngurConstructor>,
    pub fields: Vec<ZngurField>,
    pub cpp_value: Option<CppValue>,
    pub cpp_ref: Option<CppRef>,
}

#[derive(Debug)]
pub struct ZngurTrait {
    pub tr: RustTrait,
    pub methods: Vec<ZngurMethod>,
}

#[derive(Debug, Default)]
pub struct AdditionalIncludes(pub String);

#[derive(Debug, Default)]
pub struct ConvertPanicToException(pub bool);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Import {
    /// Regular file path import: import "path/to/file.zng"
    FilePath(std::path::PathBuf),
    /// Dependency-based import: import @crate-name/path/to/file.zng
    DependencyPath {
        crate_name: String,
        relative_path: std::path::PathBuf,
    },
}

impl Import {
    pub fn canonicalize(
        &self,
        current_dir: &std::path::Path,
        dep_map: Option<&DependencyMap>,
    ) -> Result<std::path::PathBuf, String> {
        match self {
            Import::FilePath(path) => match current_dir.join(path).canonicalize() {
                Ok(resolved_path) => Ok(resolved_path),
                Err(_) => Err(format!("File not found: {}", path.display())),
            },
            Import::DependencyPath {
                crate_name,
                relative_path,
            } => dep_map
                .ok_or("Dependency-based imports require cargo manifest to be provided")?
                .get_dependency_path(crate_name)
                .map(|crate_root| crate_root.join(relative_path))
                .ok_or(format!(
                    "Dependency '{}' not found in cargo metadata",
                    crate_name
                )),
        }
    }
}

#[derive(Debug, Default)]
pub struct ZngurSpec {
    pub imports: Vec<Import>,
    pub types: Vec<ZngurType>,
    pub traits: HashMap<RustTrait, ZngurTrait>,
    pub funcs: Vec<ZngurFn>,
    pub extern_cpp_funcs: Vec<ZngurExternCppFn>,
    pub extern_cpp_impls: Vec<ZngurExternCppImpl>,
    pub additional_includes: AdditionalIncludes,
    pub convert_panic_to_exception: ConvertPanicToException,
    pub cargo_manifest_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustTrait {
    Normal(RustPathAndGenerics),
    Fn {
        name: String,
        inputs: Vec<RustType>,
        output: Box<RustType>,
    },
}

impl RustTrait {
    pub fn take_assocs(mut self) -> (Self, Vec<(String, RustType)>) {
        let assocs = match &mut self {
            RustTrait::Normal(p) => std::mem::take(&mut p.named_generics),
            RustTrait::Fn { .. } => vec![],
        };
        (self, assocs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveRustType {
    Uint(u32),
    Int(u32),
    Float(u32),
    Usize,
    Bool,
    Str,
    ZngurCppOpaqueOwnedObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RustPathAndGenerics {
    pub path: Vec<String>,
    pub generics: Vec<RustType>,
    pub named_generics: Vec<(String, RustType)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustType {
    Primitive(PrimitiveRustType),
    Ref(Mutability, Box<RustType>),
    Raw(Mutability, Box<RustType>),
    Boxed(Box<RustType>),
    Slice(Box<RustType>),
    Dyn(RustTrait, Vec<String>),
    Tuple(Vec<RustType>),
    Adt(RustPathAndGenerics),
}

impl RustType {
    pub const UNIT: Self = RustType::Tuple(Vec::new());
}

impl Display for RustPathAndGenerics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let RustPathAndGenerics {
            path,
            generics,
            named_generics,
        } = self;
        for p in path {
            if p != "crate" {
                write!(f, "::")?;
            }
            write!(f, "{p}")?;
        }
        if !generics.is_empty() || !named_generics.is_empty() {
            write!(
                f,
                "::<{}>",
                generics
                    .iter()
                    .map(|x| format!("{x}"))
                    .chain(named_generics.iter().map(|x| format!("{} = {}", x.0, x.1)))
                    .join(", ")
            )?;
        }
        Ok(())
    }
}

impl Display for RustTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustTrait::Normal(tr) => write!(f, "{tr}"),
            RustTrait::Fn {
                name,
                inputs,
                output,
            } => {
                write!(f, "{name}({})", inputs.iter().join(", "))?;
                if **output != RustType::UNIT {
                    write!(f, " -> {output}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for RustType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustType::Primitive(s) => match s {
                PrimitiveRustType::Uint(s) => write!(f, "u{s}"),
                PrimitiveRustType::Int(s) => write!(f, "i{s}"),
                PrimitiveRustType::Float(s) => write!(f, "f{s}"),
                PrimitiveRustType::Usize => write!(f, "usize"),
                PrimitiveRustType::Bool => write!(f, "bool"),
                PrimitiveRustType::Str => write!(f, "str"),
                PrimitiveRustType::ZngurCppOpaqueOwnedObject => {
                    write!(f, "ZngurCppOpaqueOwnedObject")
                }
            },
            RustType::Ref(Mutability::Not, ty) => write!(f, "&{ty}"),
            RustType::Ref(Mutability::Mut, ty) => write!(f, "&mut {ty}"),
            RustType::Raw(Mutability::Not, ty) => write!(f, "*const {ty}"),
            RustType::Raw(Mutability::Mut, ty) => write!(f, "*mut {ty}"),
            RustType::Boxed(ty) => write!(f, "Box<{ty}>"),
            RustType::Tuple(v) => write!(f, "({})", v.iter().join(", ")),
            RustType::Adt(pg) => write!(f, "{pg}"),
            RustType::Dyn(tr, marker_bounds) => {
                write!(f, "dyn {tr}")?;
                for mb in marker_bounds {
                    write!(f, "+ {mb}")?;
                }
                Ok(())
            }
            RustType::Slice(s) => write!(f, "[{s}]"),
        }
    }
}

/// Specifies a package within a Cargo workspace or standalone project.
#[derive(Debug, Clone)]
pub struct PackageSpec {
    pub manifest_path: std::path::PathBuf,
    pub package_name: Option<String>,
}

impl PackageSpec {
    pub fn new(manifest_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            manifest_path: manifest_path.as_ref().to_owned(),
            package_name: None,
        }
    }

    pub fn with_package(mut self, package_name: String) -> Self {
        self.package_name = Some(package_name);
        self
    }
}

/// Maps canonical dependency names to their manifest parent directories.
///
/// A canonical dependency name is the name used by the parent project to refer to
/// it in `use` statements. If the package is renamed, the canonical name is the
/// new name. Otherwise, it is the same as the package name.
#[derive(Debug, Clone)]
pub struct DependencyMap {
    dependencies: HashMap<String, std::path::PathBuf>,
}

impl DependencyMap {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn from_package_spec(
        package_spec: &PackageSpec,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = MetadataCommand::new()
            .manifest_path(&package_spec.manifest_path)
            .exec()?;

        let package_by_name = |name: &str| metadata.packages.iter().find(|pkg| pkg.name == name);

        let target_package = if let Some(package_name) = &package_spec.package_name {
            package_by_name(package_name).ok_or(format!(
                "Package '{}' not found in cargo metadata",
                package_name
            ))?
        } else {
            metadata
                .root_package()
                .ok_or("No root package found in cargo metadata and no package specified")?
        };

        return Ok(Self {
            dependencies: target_package
                .dependencies
                .iter()
                .filter(|d| d.kind == DependencyKind::Normal)
                .filter_map(|d| {
                    package_by_name(&d.name).map(|pkg| (pkg, d.rename.as_ref().unwrap_or(&d.name)))
                })
                .map(|(pkg, rename)| (rename.clone(), pkg.manifest_path.parent().unwrap().into()))
                .collect::<HashMap<_, _>>(),
        });
    }

    pub fn from_cargo_manifest(
        manifest_path: &std::path::Path,
        package_name: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let package_spec = PackageSpec {
            manifest_path: manifest_path.to_owned(),
            package_name: package_name.map(|s| s.to_owned()),
        };
        Self::from_package_spec(&package_spec)
    }

    /// Get the root directory path for a dependency by its canonical name
    pub fn get_dependency_path(&self, crate_name: &str) -> Option<&std::path::PathBuf> {
        self.dependencies.get(crate_name)
    }
}
