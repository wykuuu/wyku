use crate::sessions::new;

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
}
