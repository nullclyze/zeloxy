use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::{ErrorKind, ProxyError};
use crate::result::ProxyResult;

/// Вспомогательная функция записи данных в поток с таймаутом
pub async fn write_all_to(stream: &mut TcpStream, buffer: Vec<u8>) -> ProxyResult<()> {
  match tokio::time::timeout(Duration::from_secs(10), stream.write_all(&buffer)).await {
    Ok(result) => match result {
      Ok(_) => ProxyResult::Ok(()),
      Err(e) => ProxyResult::Err(ProxyError::new(ErrorKind::StreamError, e.to_string())),
    },
    Err(_) => ProxyResult::Err(ProxyError::new(ErrorKind::StreamError, "failed to write buffer to stream")),
  }
}

/// Вспомогательная функция чтения данных из потока с таймаутом
pub async fn read_exact_from(stream: &mut TcpStream, buffer: &mut [u8]) -> ProxyResult<()> {
  match tokio::time::timeout(Duration::from_secs(10), stream.read_exact(buffer)).await {
    Ok(result) => match result {
      Ok(_) => ProxyResult::Ok(()),
      Err(e) => ProxyResult::Err(ProxyError::new(ErrorKind::StreamError, e.to_string())),
    },
    Err(_) => ProxyResult::Err(ProxyError::new(ErrorKind::StreamError, "failed to read buffer from stream")),
  }
}
