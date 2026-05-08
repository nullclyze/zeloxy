use crate::error::ProxyError;

/// Результат операции с прокси
pub type ProxyResult<T> = Result<T, ProxyError>;
