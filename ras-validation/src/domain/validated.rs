use ras_errors::AppError;
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone)]
pub struct Validated<T>(pub T);

#[derive(Debug)]
pub struct ValidationFailure(pub String);

impl<T> Validated<T>
where
    T: Validate + DeserializeOwned,
{
    pub fn from_json(value: serde_json::Value) -> Result<Self, AppError> {
        let inner: T =
            serde_json::from_value(value).map_err(|e| AppError::ValidationError(e.to_string()))?;
        inner
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;
        Ok(Self(inner))
    }
}

impl<T> std::ops::Deref for Validated<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
