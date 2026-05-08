/// Список ошибок прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
  Timeout,
  NotConnected,
  InvalidData,
  InvalidVersion,
  AuthFailed,
  Unsupported,
  StreamError,
  UnknownError,
}

/// Структура ошибки прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyError {
  /// Название ошибки
  kind: ErrorKind,

  /// Текст ошибки
  text: String,
}

impl ProxyError {
  /// Метод создания новой ошибки
  pub fn new(kind: ErrorKind, text: impl Into<String>) -> Self {
    Self {
      kind: kind,
      text: text.into(),
    }
  }

  /// Метод получения названия ошибки
  pub fn kind(&self) -> &ErrorKind {
    &self.kind
  }

  /// Метод получения текста ошибки
  pub fn text(&self) -> &str {
    &self.text
  }
}

impl From<std::io::Error> for ProxyError {
  fn from(error: std::io::Error) -> Self {
    Self {
      kind: ErrorKind::UnknownError,
      text: error.to_string(),
    }
  }
}
