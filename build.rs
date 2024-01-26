use std::process::Command;

fn main() {
    let version = match Command::new("git").args(["describe", "--tags", "--match=v*"]).output() {
        Ok(output) if output.status.success() =>
            String::from_utf8_lossy(&output.stdout[1..]).replacen('-', ".r", 1).replace('-', "."),
        _ => env!("CARGO_PKG_VERSION").into(),
    };
    println!("cargo:rustc-env=KIBI_VERSION={version}");

    #[cfg(feature = "embedded-syntax")]
    {
        use std::env;
        use std::ffi::OsStr;
        use std::path::{Path, PathBuf};
        use std::io::Write as _;
        use std::fs::{OpenOptions};
        use std::io::BufWriter;

        let out_dir = env::var_os("OUT_DIR").unwrap();
        let out_dir_abs = std::fs::canonicalize(&out_dir).unwrap();
        let cur_dir_abs = std::fs::canonicalize(".").unwrap();

        const SYNTAX_DIR: &str = "syntax.d";
        let syntax_dir = cur_dir_abs.join(SYNTAX_DIR);
        let syntax_files: Vec<PathBuf> = std::fs::read_dir(syntax_dir)
            .map(|syntax_dir| syntax_dir.filter_map(|f| f.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|entry| entry.path())
                .filter(|path| path.extension() == Some(OsStr::new("ini")))
                .collect())
            .unwrap_or_default();
        let dest_path = Path::new(&out_dir).join("embedded_syntax.rs");
        let mut dest = BufWriter::new(OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(dest_path)
            .expect("failed to open destination"));
        writeln!(&mut dest, "pub const SYNTAX_FILES: &[&[u8]] = &[").unwrap();
        for path in &syntax_files {
            let rel_path = pathdiff::diff_paths(path, &out_dir_abs)
                .expect("failed to calculate relative path");
            writeln!(&mut dest, r#"include_bytes!("{}"),"#, rel_path.display()).unwrap();
        }
        writeln!(&mut dest, "];").unwrap();

        println!("cargo:rerun-if-changed={SYNTAX_DIR}");
        println!("cargo:rerun-if-changed=build.rs");
    }
}
