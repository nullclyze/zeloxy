use std::time::Duration;

use tokio::net::TcpStream;

use crate::connect::*;
use crate::validate_proxy_str;
use crate::{ErrorKind, ProxyAuth, ProxyError, ProxyResult};

/// Структура прокси.
///
/// Поддерживаемые протоколы прокси:
///
/// - **HTTP** (без авторизации / с базовой авторизацией)
/// - **SOCKS4** (без авторизации / с `ident` авторизацией)
/// - **SOCKS5** (без авторизации / с `user / pass` авторизацией)
///
/// ## Примеры
///
/// ```rust, ignore
/// use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// use zeloxy::{Proxy, ProxyResult, ProxyProtocol};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём HTTP-прокси и задаём адрес целевого сервера
///   let proxy = Proxy::new("91.132.92.231:80", ProxyProtocol::Http);
///
///   match proxy.connect("example.com", 80).await {
///     ProxyResult::Ok(mut conn) => {
///       // Отправляем GET-запрос
///       conn.write_all(b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n").await?;
///
///       // Читаем ответ
///       let mut resp = Vec::new();
///       conn.read_to_end(&mut resp).await?;
///     
///       // Логгируем ответ
///       println!("{}", String::from_utf8_lossy(&resp));
///     },
///     ProxyResult::Err(_) => {}, // Просто игнорируем ошибки
///   }
///
///   Ok(())
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/nullclyze/zeloxy/tree/main/examples)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
  address: String,
  protocol: ProxyProtocol,
  timeout: u64,
  auth: Option<ProxyAuth>,
}

/// Протокол прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyProtocol {
  #[cfg(feature = "http")]
  Http,

  #[cfg(feature = "socks4")]
  Socks4,

  #[cfg(feature = "socks5")]
  Socks5,
}

impl From<&str> for ProxyProtocol {
  fn from(value: &str) -> Self {
    match value {
      "http" => Self::Http,
      "socks4" => Self::Socks4,
      "socks5" => Self::Socks5,
      _ => Self::Socks5,
    }
  }
}

impl From<String> for ProxyProtocol {
  fn from(value: String) -> Self {
    Self::from(value.as_str())
  }
}

impl Proxy {
  /// Метод создания нового прокси
  pub fn new(addr: impl Into<String>, protocol: impl Into<ProxyProtocol>) -> Self {
    Self {
      address: addr.into(),
      protocol: protocol.into(),
      timeout: 20000,
      auth: None,
    }
  }

  /// Метод создания нового прокси с авторизацией
  pub fn new_with_auth(addr: impl Into<String>, protocol: impl Into<ProxyProtocol>, auth: impl Into<ProxyAuth>) -> Self {
    Self {
      address: addr.into(),
      protocol: protocol.into(),
      timeout: 20000,
      auth: Some(auth.into()),
    }
  }

  /// Метод установки авторизации прокси
  pub fn with_auth(mut self, auth: impl Into<ProxyAuth>) -> Self {
    self.auth = Some(auth.into());
    self
  }

  /// Метод установки таймаута подключения к прокси
  pub fn with_timeout(mut self, timeout: u64) -> Self {
    self.timeout = timeout;
    self
  }

  /// Метод установки протокола прокси
  pub fn with_protocol(mut self, protocol: impl Into<ProxyProtocol>) -> Self {
    self.protocol = protocol.into();
    self
  }

  /// Метод попытки создания соединения с прокси
  pub async fn is_available(&self) -> bool {
    match tokio::time::timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.address)).await {
      Ok(result) => match result {
        Ok(_) => return true,
        Err(_) => return false,
      },
      Err(_) => return false,
    }
  }

  /// Метод получения протокола прокси
  pub fn get_protocol(&self) -> &ProxyProtocol {
    &self.protocol
  }

  /// Метод получения IP прокси
  pub fn get_ip(&self) -> String {
    self.address.split(":").collect::<Vec<&str>>()[0].to_string()
  }

  /// Метод получения IP прокси
  pub fn get_port(&self) -> u16 {
    let port_str = self.address.split(":").collect::<Vec<&str>>()[1];

    port_str.parse::<u16>().unwrap_or(match self.protocol {
      ProxyProtocol::Http => 80,
      ProxyProtocol::Socks4 => 4145,
      ProxyProtocol::Socks5 => 1080,
    })
  }

  /// Метод получения адреса прокси в формате `IP:PORT`
  pub fn get_address(&self) -> &str {
    &self.address
  }

  /// Метод получения полного адреса прокси в формате `PROTOCOL://IP:PORT`
  pub fn get_full_address(&self) -> String {
    let protocol = match self.protocol {
      #[cfg(feature = "http")]
      ProxyProtocol::Http => "http",
      #[cfg(feature = "socks4")]
      ProxyProtocol::Socks4 => "socks4",
      #[cfg(feature = "socks5")]
      ProxyProtocol::Socks5 => "socks5",
    };

    format!("{}://{}", protocol, self.address)
  }

  /// Метод подключения к прокси
  pub async fn connect(&self, target_host: impl Into<String>, target_port: u16) -> ProxyResult<TcpStream> {
    let mut stream = match tokio::time::timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.address)).await {
      Ok(result) => match result {
        Ok(s) => s,
        Err(_) => return Err(ProxyError::new(ErrorKind::NotConnected, "could not connect to specified server")),
      },
      Err(_) => {
        return Err(ProxyError::new(
          ErrorKind::Timeout,
          "failed to connect to server within specified time",
        ));
      }
    };

    match self.protocol {
      #[cfg(feature = "http")]
      ProxyProtocol::Http => connect_http(&mut stream, target_host.into(), target_port, &self.auth).await?,
      #[cfg(feature = "socks5")]
      ProxyProtocol::Socks5 => connect_socks5(&mut stream, target_host.into(), target_port, &self.auth).await?,
      #[cfg(feature = "socks4")]
      ProxyProtocol::Socks4 => connect_socks4(&mut stream, target_host.into(), target_port, &self.auth).await?,
    }

    Ok(stream)
  }

  /// Метод подключения к прокси с ранее созданным TCP-соединением
  pub async fn connect_with_stream(&self, stream: &mut TcpStream, target_host: impl Into<String>, target_port: u16) -> ProxyResult<()> {
    match self.protocol {
      #[cfg(feature = "http")]
      ProxyProtocol::Http => connect_http(stream, target_host.into(), target_port, &self.auth).await?,
      #[cfg(feature = "socks5")]
      ProxyProtocol::Socks5 => connect_socks5(stream, target_host.into(), target_port, &self.auth).await?,
      #[cfg(feature = "socks4")]
      ProxyProtocol::Socks4 => connect_socks4(stream, target_host.into(), target_port, &self.auth).await?,
    }

    Ok(())
  }
}

impl From<String> for Proxy {
  fn from(value: String) -> Self {
    if !validate_proxy_str(&value) {
      return Self::new("127.0.0.1:1080", ProxyProtocol::Socks5);
    }

    let proxy_split = value.split("://").collect::<Vec<&str>>();
    let without_protocol = proxy_split[1].split("@").collect::<Vec<&str>>();

    let (protocol, possible_auth, addr) = {
      if without_protocol.len() == 2 {
        (proxy_split[0], Some(without_protocol[0]), without_protocol[1])
      } else {
        (proxy_split[0], None, without_protocol[0])
      }
    };

    if let Some(auth) = possible_auth {
      Self::new_with_auth(addr, protocol, auth)
    } else {
      Self::new(addr, protocol)
    }
  }
}

impl From<&str> for Proxy {
  fn from(value: &str) -> Self {
    if !validate_proxy_str(value) {
      return Self::new("127.0.0.1:1080", ProxyProtocol::Socks5);
    }

    let proxy_split = value.split("://").collect::<Vec<&str>>();
    let without_protocol = proxy_split[1].split("@").collect::<Vec<&str>>();

    let (protocol, possible_auth, addr) = {
      if without_protocol.len() == 2 {
        (proxy_split[0], Some(without_protocol[0]), without_protocol[1])
      } else {
        (proxy_split[0], None, without_protocol[0])
      }
    };

    if let Some(auth) = possible_auth {
      Self::new_with_auth(addr, protocol, auth)
    } else {
      Self::new(addr, protocol)
    }
  }
}
