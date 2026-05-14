use zeloxy::{Proxy, ProxyResult, ProxyStream};

#[tokio::test]
async fn test_proxy_stream() -> ProxyResult<()> {
  let proxy = Proxy::from("socks4://68.71.242.118:4145");
  let stream = ProxyStream::new(proxy);

  stream.connect("ipinfo.io", 80).await?;

  let buf = "GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n".as_bytes();
  stream.write(buf).await?;

  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  println!("Ответ: {}", String::from_utf8_lossy(&resp));

  Ok(())
}
