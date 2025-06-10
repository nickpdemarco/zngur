use std::path::PathBuf;

use clap::Parser;
use zngur::{PackageSpec, Zngur};

#[derive(Parser)]
#[command(version)]
enum Command {
    #[command(alias = "g")]
    Generate {
        path: PathBuf,
        #[arg(long = "cargo-manifest")]
        cargo_manifest: Option<PathBuf>,
        #[arg(
            long = "package",
            requires = "cargo_manifest",
            help = "Package name to use when cargo manifest has multiple packages"
        )]
        package: Option<String>,
    },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate {
            path,
            cargo_manifest,
            package,
        } => {
            let pp = path.parent().unwrap();
            let mut builder = Zngur::from_zng_file(&path)
                .with_cpp_file(pp.join("generated.cpp"))
                .with_h_file(pp.join("generated.h"))
                .with_rs_file(pp.join("src/generated.rs"));

            if let Some(manifest_path) = cargo_manifest {
                let mut package_spec = PackageSpec::new(manifest_path);
                if let Some(package_name) = package {
                    package_spec = package_spec.with_package(package_name);
                }
                builder = builder.with_package_spec(package_spec);
            }

            builder.generate();
        }
    }
}
