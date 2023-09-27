use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("custom error: {0}")]
    Custom(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        let error = format!("{} \n{}", s, std::panic::Location::caller());
        Error::Custom(error)
    }
}
