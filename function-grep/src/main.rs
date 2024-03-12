#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(missing_debug_implementations, clippy::missing_panics_doc)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
use function_grep::{get_file_type_from_file, search_file, supported_languages};

use clap::Parser;
use std::{fs::File, io::Read, path::PathBuf};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    file: PathBuf,
    name: String,
}

#[derive(Debug)]
pub enum Error {
    CouldNotOpenFile(std::io::Error),
    CouldNotReadFile(std::io::Error),
    LibraryError(function_grep::Error),
}

///
/// # Errors
pub fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut file = File::open(&args.file).map_err(Error::CouldNotOpenFile)?;
    let file_type = get_file_type_from_file(
        &args.file.to_string_lossy(),
        supported_languages::predefined_languages(),
    )
    .map_err(Error::LibraryError)?;
    let mut code = String::new();
    file.read_to_string(&mut code)
        .map_err(Error::CouldNotReadFile)?;
    let found = search_file(&code, file_type, &args.name).map_err(Error::LibraryError)?;
    for (text, _range) in found {
        println!("{text}");
    }
    Ok(())
}
