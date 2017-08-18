use errors::anoncreds::AnoncredsError;
use errors::common::CommonError;
use errors::ledger::LedgerError;
use errors::pool::PoolError;
use errors::signus::SignusError;
use errors::wallet::WalletError;

use api::ErrorCode;
use errors::ToErrorCode;

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum IndyError {
    AnoncredsError(AnoncredsError),
    CommonError(CommonError),
    LedgerError(LedgerError),
    PoolError(PoolError),
    SignusError(SignusError),
    WalletError(WalletError),
}

impl fmt::Display for IndyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IndyError::AnoncredsError(ref err) => err.fmt(f),
            IndyError::CommonError(ref err) => err.fmt(f),
            IndyError::LedgerError(ref err) => err.fmt(f),
            IndyError::PoolError(ref err) => err.fmt(f),
            IndyError::SignusError(ref err) => err.fmt(f),
            IndyError::WalletError(ref err) => err.fmt(f)
        }
    }
}

impl error::Error for IndyError {
    fn description(&self) -> &str {
        match *self {
            IndyError::AnoncredsError(ref err) => err.description(),
            IndyError::CommonError(ref err) => err.description(),
            IndyError::LedgerError(ref err) => err.description(),
            IndyError::PoolError(ref err) => err.description(),
            IndyError::SignusError(ref err) => err.description(),
            IndyError::WalletError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            IndyError::AnoncredsError(ref err) => Some(err),
            IndyError::CommonError(ref err) => Some(err),
            IndyError::LedgerError(ref err) => Some(err),
            IndyError::PoolError(ref err) => Some(err),
            IndyError::SignusError(ref err) => Some(err),
            IndyError::WalletError(ref err) => Some(err)
        }
    }
}

impl ToErrorCode for IndyError {
    fn to_error_code(&self) -> ErrorCode {
        error!("Casting error to ErrorCode: {}", self);
        match *self {
            IndyError::AnoncredsError(ref err) => err.to_error_code(),
            IndyError::CommonError(ref err) => err.to_error_code(),
            IndyError::LedgerError(ref err) => err.to_error_code(),
            IndyError::PoolError(ref err) => err.to_error_code(),
            IndyError::SignusError(ref err) => err.to_error_code(),
            IndyError::WalletError(ref err) => err.to_error_code()
        }
    }
}

impl From<AnoncredsError> for IndyError {
    fn from(err: AnoncredsError) -> IndyError {
        IndyError::AnoncredsError(err)
    }
}

impl From<CommonError> for IndyError {
    fn from(err: CommonError) -> IndyError {
        IndyError::CommonError(err)
    }
}

impl From<PoolError> for IndyError {
    fn from(err: PoolError) -> IndyError {
        IndyError::PoolError(err)
    }
}

impl From<WalletError> for IndyError {
    fn from(err: WalletError) -> IndyError {
        IndyError::WalletError(err)
    }
}

impl From<LedgerError> for IndyError {
    fn from(err: LedgerError) -> IndyError {
        IndyError::LedgerError(err)
    }
}

impl From<SignusError> for IndyError {
    fn from(err: SignusError) -> IndyError {
        IndyError::SignusError(err)
    }
}