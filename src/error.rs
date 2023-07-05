use core::fmt::{Debug, Formatter};

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    IO(&'static str),
    Format(&'static str),
}

impl Error {
    fn field_name(&self) -> &'static str {
        match self {
            Error::IO(_) => "io",
            Error::Format(_) => "format",
        }
    }

    fn field_data(&self) -> &'static str {
        match self {
            Error::IO(data) => data,
            Error::Format(data) => data,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Error")
            .field(self.field_name(), &self.field_data())
            .finish()
    }
}
