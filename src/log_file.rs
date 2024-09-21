use std::{
    fs::{File, OpenOptions},
    io::{Result, Write},
    path::Path,
};

pub(crate) fn new(header: Option<&str>, path: &Path) -> Result<File> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    if let Some(header) = header {
        writeln!(file, "{header}")?;
    }
    Ok(file)
}
