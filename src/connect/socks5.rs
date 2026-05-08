use bytes::{BufMut, BytesMut};
use tokio::net::TcpStream;

use crate::rw::{read_exact_from, write_all_to};
use crate::{ErrorKind, ProxyAuth, ProxyError, ProxyResult};

const PROXY_VERSION: u8 = 0x05;
const USER_PASS_AUTH: u8 = 0x01;
const CONNECT_COMMAND: u8 = 0x01;

/// Метод авторизации SOCKS5 прокси
#[derive(Debug, PartialEq, Eq)]
enum AuthMethod {
  NoAuth,
  UserPass,
  Other,
}

impl AuthMethod {
  pub fn from(id: u8) -> Self {
    match id {
      0x00 => Self::NoAuth,
      0x02 => Self::UserPass,
      _ => Self::Other,
    }
  }
}

/// Статус ответа SOCKS5 прокси
#[derive(Debug, PartialEq, Eq)]
enum ResponseStatus {
  Success,
  ServerError,
  ConnectionDenied,
  NetworkUnavailable,
  HostUnavailable,
  ConnectionRefused,
  TimeToLiveExpired,
  ProtocolError,
  UnsupportedAddrType,
  UnknownResponse,
}

impl ResponseStatus {
  pub fn from(id: u8) -> Self {
    match id {
      0x00 => Self::Success,
      0x01 => Self::ServerError,
      0x02 => Self::ConnectionDenied,
      0x03 => Self::NetworkUnavailable,
      0x04 => Self::HostUnavailable,
      0x05 => Self::ConnectionRefused,
      0x06 => Self::TimeToLiveExpired,
      0x07 => Self::ProtocolError,
      0x08 => Self::UnsupportedAddrType,
      _ => Self::UnknownResponse,
    }
  }
}

/// Метод создания подключения с SOCKS5 прокси
pub async fn connect_socks5(stream: &mut TcpStream, target_host: String, target_port: u16, auth: &Option<ProxyAuth>) -> ProxyResult<()> {
  let greet = if auth.is_some() {
    vec![PROXY_VERSION, 0x02, 0x00, USER_PASS_AUTH]
  } else {
    vec![PROXY_VERSION, 0x01, 0x00]
  };

  write_all_to(stream, greet).await?;

  let mut response = [0u8; 2];

  read_exact_from(stream, &mut response).await?;

  if response[0] != PROXY_VERSION {
    return Err(ProxyError::new(ErrorKind::InvalidVersion, "invalid proxy version in response"));
  }

  match AuthMethod::from(response[1]) {
    AuthMethod::NoAuth => {}
    AuthMethod::UserPass => {
      if let Some(auth) = auth {
        let username = auth.username();
        let password = auth.password();

        if username.len() > 255 || password.len() > 255 {
          return Err(ProxyError::new(ErrorKind::InvalidData, "username or password is too long"));
        }

        let mut buffer = BytesMut::with_capacity(2 + username.len() + password.len());
        buffer.put_u8(0x01);
        buffer.put_u8(username.len() as u8);
        buffer.put_slice(username.as_bytes());
        buffer.put_u8(password.len() as u8);
        buffer.put_slice(password.as_bytes());

        write_all_to(stream, buffer.into()).await?;

        let mut resp = [0u8; 2];

        read_exact_from(stream, &mut resp).await?;

        if resp[0] != 0x01 {
          return Err(ProxyError::new(ErrorKind::AuthFailed, "invalid authorization version"));
        }

        if resp[1] != 0x00 {
          return Err(ProxyError::new(
            ErrorKind::AuthFailed,
            "authorization failed (possibly incorrect password or username)",
          ));
        }
      } else {
        return Err(ProxyError::new(
          ErrorKind::AuthFailed,
          "proxy requires authorization (username, password)",
        ));
      }
    }
    AuthMethod::Other => return Err(ProxyError::new(ErrorKind::Unsupported, "unsupported authorization method")),
  }

  let mut req = BytesMut::with_capacity(512);
  req.put_u8(PROXY_VERSION);
  req.put_u8(CONNECT_COMMAND);
  req.put_u8(0x00); // Зарезервированный байт

  if let Ok(ipv4) = target_host.parse::<std::net::Ipv4Addr>() {
    req.put_u8(0x01); // IPv4 адрес
    req.put_slice(&ipv4.octets());
  } else if let Ok(ipv6) = target_host.parse::<std::net::Ipv6Addr>() {
    req.put_u8(0x04); // IPv6 адрес
    req.put_slice(&ipv6.octets());
  } else {
    req.put_u8(0x03); // Домен
    let host_bytes = target_host.as_bytes();

    if host_bytes.len() > 255 {
      return Err(ProxyError::new(ErrorKind::InvalidData, "target host is too long"));
    }

    req.put_u8(host_bytes.len() as u8);
    req.put_slice(host_bytes);
  }

  req.put_u16(target_port);

  write_all_to(stream, req.into()).await?;

  let mut resp = [0u8; 4];

  read_exact_from(stream, &mut resp).await?;

  if resp[0] != PROXY_VERSION {
    return Err(ProxyError::new(ErrorKind::InvalidVersion, "invalid proxy version in response"));
  }

  let response_status = ResponseStatus::from(resp[1]);

  if response_status != ResponseStatus::Success {
    return Err(ProxyError::new(
      ErrorKind::NotConnected,
      format!("unsuccessful proxy response status: {:?}", response_status),
    ));
  }

  let atyp = resp[3];

  match atyp {
    0x01 => {
      let mut addr = [0u8; 6];
      read_exact_from(stream, &mut addr).await?;
    }
    0x04 => {
      let mut addr = [0u8; 18];
      read_exact_from(stream, &mut addr).await?;
    }
    0x03 => {
      let mut len = [0u8; 1];
      read_exact_from(stream, &mut len).await?;
      let mut rest = vec![0u8; len[0] as usize + 2];
      read_exact_from(stream, &mut rest).await?;
    }
    _ => {
      return Err(ProxyError::new(
        ErrorKind::InvalidData,
        format!("unknown address type in reply: 0x{:02x}", atyp),
      ));
    }
  }

  Ok(())
}
