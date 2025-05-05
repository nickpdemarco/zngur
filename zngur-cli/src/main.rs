use std::path::PathBuf;

use clap::Parser;
use zngur::Zngur;

#[derive(Parser)]
#[command(version)]
enum Command {
    #[command(alias = "g")]
    Generate { path: PathBuf },
    #[command(alias = "gm")]
    GenerateMerged {
        paths: Vec<PathBuf>,
        #[arg(long)]
        r#in: PathBuf,
    },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate { path } => {
            let pp = path.parent().unwrap();
            Zngur::from_zng_file(&path)
                .with_cpp_file(pp.join("generated.cpp"))
                .with_h_file(pp.join("generated.h"))
                .with_rs_file(pp.join("src/generated.rs"))
                .generate();
        }
        Command::GenerateMerged {
            paths,
            r#in: in_dir,
        } => {
            Zngur::from_zng_files(paths)
                .with_cpp_file(in_dir.join("generated.cpp"))
                .with_h_file(in_dir.join("generated.h"))
                .with_rs_file(in_dir.join("src/generated.rs"))
                .generate_merged();
        }
    }
}
