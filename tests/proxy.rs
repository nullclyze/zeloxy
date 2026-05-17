use std::io::{Error, ErrorKind};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{Proxy, ProxyResult, ProxyType};

#[test]
fn test_proxy_from() {
  let basic_socks5_proxy = Proxy::from("socks5://212.58.132.5:1080");
  let socks5_proxy_with_auth = Proxy::from("socks5://user:pass@212.58.132.5:1080");
  let socks4_proxy_with_auth = Proxy::from("socks4://user_id@68.71.242.118:4145");
  let wrong_proxy = Proxy::from("socks4://68.71.242.118");

  println!("{:?}", basic_socks5_proxy);
  println!("{:?}", socks5_proxy_with_auth);
  println!("{:?}", socks4_proxy_with_auth);
  println!("{:?}", wrong_proxy);
}

#[tokio::test]
async fn test_http_proxy() -> std::io::Result<()> {
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http);

  let mut conn = match proxy.connect("ipinfo.io", 80).await {
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
  let proxy = Proxy::new("212.58.132.5:1080", ProxyType::Socks5);

  let mut conn = match proxy.connect("ipinfo.io", 80).await {
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
  let proxy = Proxy::new("68.71.242.118:4145", ProxyType::Socks4);

  let mut conn = match proxy.connect("ipinfo.io", 80).await {
    ProxyResult::Ok(s) => s,
    ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
  };

  conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

  let mut buf = Vec::new();
  conn.read_to_end(&mut buf).await?;

  println!("{}", String::from_utf8_lossy(&buf));

  Ok(())
}
