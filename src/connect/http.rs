use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::rw::write_all_to;
use crate::{ErrorKind, ProxyAuth, ProxyError, ProxyResult};

/// Метод создания подключения с HTTP прокси
pub async fn connect_http(stream: &mut TcpStream, target_host: String, target_port: u16, auth: &Option<ProxyAuth>) -> ProxyResult<()> {
  let mut req = format!(
    "GET http://{}:{} HTTP/1.1\r\n\
    Host: {}\r\n",
    target_host, target_port, target_host
  );

  if let Some(auth) = auth {
    let credentials = format!("{}:{}", auth.username(), auth.password());
    let encoded = STANDARD.encode(&credentials);

    req.push_str(&format!("Proxy-Authorization: Basic {}\r\n", encoded));
  }

  req.push_str("\r\n");

  write_all_to(stream, req.into()).await?;

  let mut resp = vec![0; 8192];
  let n = stream.read(&mut resp).await?;
  resp.truncate(n);

  let resp_str = String::from_utf8_lossy(&resp);
  let status_line = resp_str.lines().next().unwrap_or("");

  if !status_line.contains("200") && !status_line.contains("Connection established") {
    return Err(ProxyError::new(
      ErrorKind::NotConnected,
      format!("unsuccessful proxy response status: {}", status_line),
    ));
  }

  Ok(())
}
