use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::connect::{connect_http, connect_socks4, connect_socks5};
use crate::{ErrorKind, ProxyAuth, ProxyError, ProxyResult};

/// Структура прокси.
///
/// Поддерживаемые прокси:
///
/// - **HTTP** (без авторизации / с базовой авторизацией)
/// - **SOCKS4** (без авторизации / с `ident` авторизацией)
/// - **SOCKS5** (без авторизации / с `user / pass` авторизацией)
///
/// ## Примеры
///
/// ```rust, ignore
/// use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// use zeloxy::{Proxy, ProxyResult, ProxyType};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём HTTP-прокси и задаём адрес целевого сервера
///   let proxy = Proxy::new("PROXY_IP:PROXY_PORT", ProxyType::Http)
///     .bind("example.com".to_string(), 80);
///
///   match proxy.connect().await {
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
#[derive(Debug)]
pub struct Proxy {
  proxy_type: ProxyType,
  proxy_address: String,
  target: RwLock<TargetServer>,
  timeout: u64,
  auth: Option<ProxyAuth>,
}

/// Тип прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
  Http,
  Socks5,
  Socks4,
}

/// Структура адреса целевого сервера
#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetServer {
  host: Option<String>,
  port: Option<u16>,
}

impl Default for TargetServer {
  fn default() -> Self {
    Self { host: None, port: None }
  }
}

impl From<String> for Proxy {
  fn from(value: String) -> Self {
    let split = value.split("://").collect::<Vec<&str>>();
    let (protocol, proxy) = (split.get(0).unwrap_or(&"socks5"), split.get(1).unwrap_or(&"127.0.0.1"));

    Self {
      proxy_address: (*proxy).to_string(),
      proxy_type: match *protocol {
        "http" => ProxyType::Http,
        "socks5" => ProxyType::Socks5,
        "socks4" => ProxyType::Socks4,
        _ => ProxyType::Socks5,
      },
      target: RwLock::new(TargetServer::default()),
      timeout: 20000,
      auth: None,
    }
  }
}

impl From<&str> for Proxy {
  fn from(value: &str) -> Self {
    let split = value.split("://").collect::<Vec<&str>>();
    let (protocol, proxy) = (split.get(0).unwrap_or(&"socks5"), split.get(1).unwrap_or(&"127.0.0.1"));

    Self {
      proxy_address: (*proxy).to_string(),
      proxy_type: match *protocol {
        "http" => ProxyType::Http,
        "socks5" => ProxyType::Socks5,
        "socks4" => ProxyType::Socks4,
        _ => ProxyType::Socks5,
      },
      target: RwLock::new(TargetServer::default()),
      timeout: 20000,
      auth: None,
    }
  }
}

impl Clone for Proxy {
  fn clone(&self) -> Self {
    Self {
      proxy_type: self.proxy_type.clone(),
      proxy_address: self.proxy_address.clone(),
      target: {
        if let Some((host, port)) = self.get_target_server() {
          RwLock::new(TargetServer {
            host: Some(host),
            port: Some(port),
          })
        } else {
          RwLock::new(TargetServer::default())
        }
      },
      timeout: self.timeout,
      auth: self.auth.clone(),
    }
  }
}

impl Proxy {
  /// Метод создания нового прокси
  pub fn new(proxy_address: impl Into<String>, proxy_type: ProxyType) -> Self {
    Self {
      proxy_address: proxy_address.into(),
      proxy_type: proxy_type,
      target: RwLock::new(TargetServer::default()),
      timeout: 20000,
      auth: None,
    }
  }

  /// Метод создания нового прокси с авторизацией
  pub fn new_with_auth(proxy_address: impl Into<String>, proxy_type: ProxyType, auth: ProxyAuth) -> Self {
    Self {
      proxy_address: proxy_address.into(),
      proxy_type: proxy_type,
      target: RwLock::new(TargetServer::default()),
      timeout: 20000,
      auth: Some(auth),
    }
  }

  /// Метод установки адреса целевого сервера
  pub fn bind(self, target_host: impl Into<String>, target_port: u16) -> Self {
    match self.target.try_write() {
      Ok(mut g) => {
        g.host = Some(target_host.into());
        g.port = Some(target_port);
      }
      Err(_) => {}
    }

    self
  }

  /// Метод переустановки адреса целевого сервера
  pub fn rebind(&self, target_host: impl Into<String>, target_port: u16) {
    match self.target.try_write() {
      Ok(mut g) => {
        g.host = Some(target_host.into());
        g.port = Some(target_port);
      }
      Err(_) => {}
    }
  }

  /// Метод установки таймаута подключения к прокси
  pub fn set_timeout(mut self, timeout: u64) -> Self {
    self.timeout = timeout;
    self
  }

  /// Метод установки типа прокси
  pub fn set_proxy_type(mut self, proxy_type: ProxyType) -> Self {
    self.proxy_type = proxy_type;
    self
  }

  /// Метод попытки создания соединения с прокси
  pub async fn is_available(&self) -> bool {
    match timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.proxy_address)).await {
      Ok(result) => match result {
        Ok(_) => return true,
        Err(_) => return false,
      },
      Err(_) => return false,
    }
  }

  /// Метод получения IP прокси
  pub fn get_ip(&self) -> Option<String> {
    if let Some(ip) = self.proxy_address.split(":").collect::<Vec<&str>>().get(0) {
      Some(ip.to_string())
    } else {
      None
    }
  }

  /// Метод получения адреса целевого сервера в формате `IP:PORT`
  pub fn get_target_server(&self) -> Option<(String, u16)> {
    match self.target.try_read() {
      Ok(g) => {
        let Some(host) = g.host.clone() else {
          return None;
        };
        let Some(port) = g.port else {
          return None;
        };

        Some((host, port))
      }
      Err(_) => None,
    }
  }

  /// Метод подключения к прокси
  pub async fn connect(&self) -> ProxyResult<TcpStream> {
    let (target_host, target_port) = if let Some(target_addr) = self.get_target_server() {
      target_addr
    } else {
      return Err(ProxyError::new(ErrorKind::InvalidData, "target server address is not specified"));
    };

    let mut stream = match timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.proxy_address)).await {
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

    match self.proxy_type {
      ProxyType::Http => connect_http(&mut stream, target_host, target_port, &self.auth).await?,
      ProxyType::Socks5 => connect_socks5(&mut stream, target_host, target_port, &self.auth).await?,
      ProxyType::Socks4 => connect_socks4(&mut stream, target_host, target_port, &self.auth).await?,
    }

    Ok(stream)
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Error, ErrorKind};

  use tokio::io::{AsyncReadExt, AsyncWriteExt};

  use crate::result::ProxyResult;
  use crate::{Proxy, ProxyType};

  #[tokio::test]
  async fn test_http_proxy() -> std::io::Result<()> {
    let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http).bind("ipinfo.io", 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }

  #[tokio::test]
  async fn test_socks5_proxy() -> std::io::Result<()> {
    let proxy = Proxy::new("212.58.132.5:1080", ProxyType::Socks5).bind("ipinfo.io", 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }

  #[tokio::test]
  async fn test_socks4_proxy() -> std::io::Result<()> {
    let proxy = Proxy::new("68.71.242.118:4145", ProxyType::Socks4).bind("ipinfo.io", 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }
}
