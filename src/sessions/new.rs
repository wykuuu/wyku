use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn new(app: &str) -> io::Result<PathBuf> {
    let path = &dirs::data_dir()
        .ok_or_else(|| io::Error::other("data directory not found"))?
        .join("wyku")
        .join("sessions");

    loop {
        match try_create(path, app) {
            Ok(path) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
    }
}

fn try_create(path: &PathBuf, name: &str) -> io::Result<PathBuf> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_millis()
        .to_string();

    let session = path.join(name).join(ms);

    fs::create_dir_all(path.join(name))?;
    fs::create_dir(&session)?;

    Ok(session)
}
