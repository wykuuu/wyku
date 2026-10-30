use crate::sessions::{list, new};

use std::io;
use std::path::PathBuf;

pub(crate) const VERSION: u8 = 1;

pub(crate) struct Session {
    path: PathBuf,
}

impl Session {
    pub(crate) fn new(app: &str) -> io::Result<Self> {
        Ok(Session {
            path: new::new(app)?,
        })
    }

    pub(crate) fn list(app: Option<&str>) -> io::Result<Vec<Self>> {
        Ok(list::list(app)?
            .into_iter()
            .map(|path| Session { path })
            .collect())
    }
}
