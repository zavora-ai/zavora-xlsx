use napi::Error as NapiError;
use napi::Status;

/// Convert a zavora_xlsx::Error into a napi::Error.
pub fn to_napi_error(e: zavora_xlsx::Error) -> NapiError {
    NapiError::new(Status::GenericFailure, format!("{e}"))
}

/// Shorthand: convert Result<T, zavora_xlsx::Error> to napi::Result<T>.
pub trait IntoNapi<T> {
    fn into_napi(self) -> napi::Result<T>;
}

impl<T> IntoNapi<T> for Result<T, zavora_xlsx::Error> {
    fn into_napi(self) -> napi::Result<T> {
        self.map_err(to_napi_error)
    }
}
