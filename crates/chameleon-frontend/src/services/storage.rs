use gloo::storage::{errors::StorageError, LocalStorage, SessionStorage, Storage};
use uuid::Uuid;

pub fn local_id() -> Result<String, StorageError> {
    const KEY: &str = "local-id";
    match LocalStorage::get(KEY) {
        Ok(values) => Ok(values),
        Err(StorageError::KeyNotFound(_)) => {
            LocalStorage::set(KEY, Uuid::new_v4().to_string())?;
            LocalStorage::get(KEY)
        }
        Err(err) => Err(err),
    }
}

pub fn session_id() -> Result<String, StorageError> {
    const KEY: &str = "session-id";
    match SessionStorage::get(KEY) {
        Ok(values) => Ok(values),
        Err(StorageError::KeyNotFound(_)) => {
            SessionStorage::set(KEY, Uuid::new_v4().to_string())?;
            SessionStorage::get(KEY)
        }
        Err(err) => Err(err),
    }
}
