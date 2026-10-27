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

#[cfg(test)]
mod test {
    use super::*;

    fn app_dir(app: &str) -> PathBuf {
        dirs::data_dir()
            .unwrap()
            .join("wyku")
            .join("sessions")
            .join(app)
    }

    #[test]
    fn empty_session() {
        let app = "wyku-test-empty";
        let path = new(app).unwrap();

        assert!(path.is_dir());
        assert!(fs::read_dir(&path).unwrap().next().is_none());
        assert_eq!(path.parent().unwrap(), app_dir(app));
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u128>()
            .unwrap();
    }

    #[test]
    fn two_sessions() {
        let app = "wyku-test-two";

        let first = new(app).unwrap();
        let second = new(app).unwrap();

        assert_ne!(first, second);
        assert!(first.is_dir());
        assert!(second.is_dir());
        assert!(fs::read_dir(&first).unwrap().next().is_none());
        assert!(fs::read_dir(&second).unwrap().next().is_none());
        assert_eq!(first.parent().unwrap(), app_dir(app));
        assert_eq!(second.parent().unwrap(), app_dir(app));
    }
}
