use std::fs;
use std::io;
use std::path::PathBuf;

use directories::ProjectDirs;

const APPLICATION_NAME: &str = "zygo";
const DATABASE_FILE_NAME: &str = "zygo.db";

fn application_data_dir() -> io::Result<PathBuf> {
    ProjectDirs::from("", "", APPLICATION_NAME)
        .map(|project_dirs| project_dirs.data_local_dir().to_path_buf())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine the application data directory",
            )
        })
}

pub fn database_path() -> io::Result<PathBuf> {
    let data_dir = application_data_dir()?;
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join(DATABASE_FILE_NAME))
}

pub fn delete_database() -> io::Result<bool> {
    let path = database_path()?;
    let mut deleted = remove_file_if_exists(&path)?;

    for suffix in ["-shm", "-wal"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        deleted |= remove_file_if_exists(&PathBuf::from(sidecar))?;
    }

    Ok(deleted)
}

fn remove_file_if_exists(path: &std::path::Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
