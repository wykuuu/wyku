use std::fs;
use std::io;
use std::path::PathBuf;

use crate::sessions::session::VERSION;

pub(crate) fn list(app: Option<&str>) -> io::Result<Vec<PathBuf>> {
    let path = dirs::data_dir()
        .ok_or_else(|| io::Error::other("data directory not found"))?
        .join("wyku")
        .join("sessions")
        .join(app.unwrap_or_default())
        .join(VERSION.to_string());

    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let mut sessions = Vec::new();

    for entry in entries {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let Some(id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u128>().ok())
        else {
            continue;
        };

        sessions.push((id, entry.path()));
    }

    sessions.sort_unstable_by(|left, right| right.0.cmp(&left.0));

    Ok(sessions.into_iter().map(|(_, path)| path).collect())
}
