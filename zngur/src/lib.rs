//! This crate contains an API for using the Zngur code generator inside build scripts. For more information
//! about the Zngur itself, see [the documentation](https://hkalbasi.github.io/zngur).

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

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
    zng_files: Vec<PathBuf>,
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_files: vec![zng_file_path.as_ref().to_owned()],
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
        }
    }

    pub fn from_zng_files(zng_file_paths: Vec<impl AsRef<Path>>) -> Self {
        Zngur {
            zng_files: zng_file_paths
                .into_iter()
                .map(|p| p.as_ref().to_owned())
                .collect(),
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
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

    fn emit(self, generator: ZngurGenerator) {
        let (rust, h, cpp) = generator.render();
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

    pub fn generate(mut self) {
        assert_eq!(
            self.zng_files.len(),
            1,
            "Exactly one zng file must be provided"
        );
        let file = ZngurGenerator::build_from_zng(ParsedZngFile::parse(std::mem::take(
            &mut self.zng_files[0],
        )));

        self.emit(file);
    }

    pub fn generate_merged(mut self) {
        let mut zngur = zngur_generator::ZngurSpec::default();
        for path in std::mem::take(&mut self.zng_files) {
            ParsedZngFile::parse_into(&mut zngur, path);
        }
        let file = ZngurGenerator::build_from_zng(zngur);
        self.emit(file);
    }
}
