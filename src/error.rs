use std::fmt::{Display, Formatter, Result as FmtResult};
use std::num::ParseIntError;

use log::SetLoggerError;
use rusoto_core::{CredentialsError, ParseRegionError, TlsError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl Display for Error {
    fn fmt(&self, mut f: &mut Formatter) -> FmtResult {
        self.0.fmt(&mut f)
    }
}

macro_rules! impl_from_error {
    ($error:ty) => {
        impl From<$error> for Error {
            fn from(error: $error) -> Self {
                Error(error.to_string())
            }
        }
    }
}

impl_from_error!(CredentialsError);
impl_from_error!(ParseIntError);
impl_from_error!(ParseRegionError);
impl_from_error!(SetLoggerError);
impl_from_error!(TlsError);
