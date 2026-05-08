use bytes::{BufMut, BytesMut};
use tokio::net::TcpStream;

use crate::rw::{read_exact_from, write_all_to};
use crate::{ErrorKind, ProxyAuth, ProxyError, ProxyResult};

const PROXY_VERSION: u8 = 0x04;
const CONNECT_COMMAND: u8 = 0x01;

/// Статус ответа SOCKS4 прокси
#[derive(Debug, PartialEq, Eq)]
enum ResponseStatus {
  Success,
  RequestRejected,
  IdentdError,
  AuthError,
  UnknownResponse,
}

impl ResponseStatus {
  pub fn from(id: u8) -> Self {
    match id {
      0x5a => Self::Success,
      0x5b => Self::RequestRejected,
      0x5c => Self::IdentdError,
      0x5d => Self::AuthError,
      _ => Self::UnknownResponse,
    }
  }
}

/// Метод создания подключения с SOCKS4 прокси
pub async fn connect_socks4(stream: &mut TcpStream, target_host: String, target_port: u16, auth: &Option<ProxyAuth>) -> ProxyResult<()> {
  let mut req = BytesMut::with_capacity(512);
  req.put_u8(PROXY_VERSION);
  req.put_u8(CONNECT_COMMAND);
  req.put_u16(target_port);

  if let Ok(ipv4) = target_host.parse::<std::net::Ipv4Addr>() {
    req.put_slice(&ipv4.octets());

    if let Some(auth) = auth {
      req.put_slice(auth.username().as_bytes());
    } else {
      req.put_u8(0x00);
    }
  } else {
    req.put_slice(&[0x00, 0x00, 0x00, 0x01]);

    if let Some(auth) = auth {
      req.put_slice(auth.username().as_bytes());
    } else {
      req.put_u8(0x00);
    }

    if target_host.len() > 255 {
      return Err(ProxyError::new(ErrorKind::InvalidData, "target host is too long"));
    }

    req.put_slice(target_host.as_bytes());
    req.put_u8(0x00);
  }

  write_all_to(stream, req.into()).await?;

  let mut resp = [0u8; 8];

  read_exact_from(stream, &mut resp).await?;

  // В SOCKS4 здесь обычно всегда 0x00
  if resp[0] != 0x00 {
    return Err(ProxyError::new(ErrorKind::InvalidData, "invalid proxy response"));
  }

  let response_status = ResponseStatus::from(resp[1]);

  if response_status != ResponseStatus::Success {
    return Err(ProxyError::new(
      ErrorKind::NotConnected,
      format!("unsuccessful proxy response status: {:?}", response_status),
    ));
  }

  Ok(())
}
