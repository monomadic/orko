use sass_rs;

use std;
use std::path::{Path, PathBuf};
use std::io;

use templar;
use templar::{TemplateContext, Node};


#[derive(Debug)]
pub struct ProcessedFile {
    pub source: PathBuf,
    // pub action: BuildAction,
    pub result: Result<PathBuf, BuildErrorReason>,
}

// build error should probably have some file params ... be a struct with a reason field
#[derive(Debug)]
pub enum BuildErrorReason {
    IO(io::Error),
    Sass(String),
    // TemplarParse(templar::parse::ParseError),
    // TemplarWrite(templar::output::WriteError<DirectiveError>),
    // UTF8Error(std::string::FromUtf8Error),
}

/// Returns valid site directories give a path.
pub fn build_path(path:&Path) -> bool {
    let path = path.iter().last().expect("a last component in a path");
    if let Some(path_str) = path.to_str() {
        !(path_str.starts_with(".") || path_str.starts_with("_"))
    } else {
        false
    }
}

pub fn build(source: &Path, destination: &Path) -> io::Result<Vec<ProcessedFile>> {
    std::fs::create_dir_all(destination)?;
    let paths = read_directory_paths(source)?;
    println!("sites {:?}", paths);

    Ok(paths.into_iter().flat_map(|path| {
        println!("site {:?}", path);
        if build_path(&path) {
            vec![]
        } else {
            vec![ProcessedFile {
                source: path,
                // action: BuildAction::Ignore,
                result: Ok(destination.to_path_buf()),
            }]
        }
    }).collect())
}

pub fn read_directory_paths(path:&Path) -> io::Result<Vec<PathBuf>> {
    let mut paths : Vec<PathBuf> = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path().to_path_buf();
        paths.push(file_path);
    }

    Ok(paths)
}
