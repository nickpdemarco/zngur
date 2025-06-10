//! This crate contains an API for using the Zngur code generator inside build scripts. For more information
//! about the Zngur itself, see [the documentation](https://hkalbasi.github.io/zngur).

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub use zngur_def::{DependencyMap, PackageSpec};
use zngur_generator::{ParsedZngFile, ZngurGenerator};

#[must_use]
/// Builder for the Zngur generator.
///
/// Usage:
/// ```ignore
/// let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
/// let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
/// Zngur::from_zng_file(crate_dir.join("main.zng"))
///     .with_cpp_file(out_dir.join("generated.cpp"))
///     .with_h_file(out_dir.join("generated.h"))
///     .with_rs_file(out_dir.join("generated.rs"))
///     .generate();
/// ```
pub struct Zngur {
    zng_file: PathBuf,
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
    package_spec: Option<PackageSpec>,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_file: zng_file_path.as_ref().to_owned(),
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
            package_spec: None,
        }
    }

    pub fn with_h_file(mut self, path: impl AsRef<Path>) -> Self {
        self.h_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_cpp_file(mut self, path: impl AsRef<Path>) -> Self {
        self.cpp_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_rs_file(mut self, path: impl AsRef<Path>) -> Self {
        self.rs_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_package_spec(mut self, package_spec: PackageSpec) -> Self {
        self.package_spec = Some(package_spec);
        self
    }

    pub fn with_cargo_manifest(mut self, path: Option<impl AsRef<Path>>) -> Self {
        self.package_spec = path.map(|p| PackageSpec::new(p.as_ref()));
        self
    }

    pub fn generate(self) {
        let dep_map = self
            .package_spec
            .map(|spec| {
                DependencyMap::from_package_spec(&spec)
                    .inspect_err(|e| {
                        eprintln!("Warning: Failed to parse cargo metadata: {}", e);
                    })
                    .ok()
            })
            .flatten();

        let file =
            ZngurGenerator::build_from_zng(ParsedZngFile::parse(self.zng_file, dep_map.as_ref()));

        let (rust, h, cpp) = file.render();
        let rs_file_path = self.rs_file_path.expect("No rs file path provided");
        let h_file_path = self.h_file_path.expect("No h file path provided");
        File::create(rs_file_path)
            .unwrap()
            .write_all(rust.as_bytes())
            .unwrap();
        File::create(h_file_path)
            .unwrap()
            .write_all(h.as_bytes())
            .unwrap();
        if let Some(cpp) = cpp {
            let cpp_file_path = self.cpp_file_path.expect("No cpp file path provided");
            File::create(cpp_file_path)
                .unwrap()
                .write_all(cpp.as_bytes())
                .unwrap();
        }
    }
}
