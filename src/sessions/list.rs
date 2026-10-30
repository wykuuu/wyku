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

#[cfg(test)]
mod test {
    use super::*;

    const PREFIX: &str = "wykuuuuuuuuuuuuuuuuuuuuuuuuuuuu";

    fn version_dir(app: &str) -> PathBuf {
        dirs::data_dir()
            .unwrap()
            .join("wyku")
            .join("sessions")
            .join(app)
            .join(VERSION.to_string())
    }

    #[test]
    fn missing_dir_is_empty() {
        let app = format!("{PREFIX}-empty");

        assert!(list(Some(&app)).unwrap().is_empty());
    }

    #[test]
    fn version_path_is_a_file() {
        let app = format!("{PREFIX}-file");
        let version = version_dir(&app);

        fs::create_dir_all(version.parent().unwrap()).unwrap();
        fs::write(&version, b"").unwrap();

        let error = list(Some(&app)).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotADirectory);
    }

    #[test]
    fn filters_and_sorts_newest_first() {
        let app = format!("{PREFIX}-sort");
        let version = version_dir(&app);
        fs::create_dir_all(&version).unwrap();

        for name in ["2", "10", "100", "nope"] {
            fs::create_dir_all(version.join(name)).unwrap();
        }
        fs::write(version.join("3"), b"").unwrap();

        let sessions = list(Some(&app)).unwrap();

        assert_eq!(sessions, ["100", "10", "2"].map(|name| version.join(name)));
    }
}
