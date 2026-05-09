/// Структура опций GET-запроса
pub struct GetRequestOpts {
  pub user_agent: Option<String>,
  pub accept: Option<String>,
  pub accept_language: Option<String>,
  pub accept_encoding: Option<String>,
  pub connection: Option<String>,
}

impl Default for GetRequestOpts {
  fn default() -> Self {
    Self {
      user_agent: None,
      accept: None,
      accept_language: None,
      accept_encoding: None,
      connection: None,
    }
  }
}

impl GetRequestOpts {
  /// Метод создания строки с опциями для запроса
  pub fn to_request(&self) -> String {
    let mut opts = String::new();

    if let Some(user_agent) = &self.user_agent {
      opts.push_str(&format!("User-Agent: {}", user_agent));
    }

    if let Some(accept) = &self.accept {
      opts.push_str(&format!("\r\nAccept: {}", accept));
    }

    if let Some(accept_language) = &self.accept_language {
      opts.push_str(&format!("\r\nAccept-Language: {}", accept_language));
    }

    if let Some(accept_encoding) = &self.accept_encoding {
      opts.push_str(&format!("\r\nAccept-Encoding: {}", accept_encoding));
    }

    if let Some(connection) = &self.connection {
      opts.push_str(&format!("\r\nConnection: {}", connection));
    }

    opts
  }
}
