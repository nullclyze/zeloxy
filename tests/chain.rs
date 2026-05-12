#[cfg(test)]
mod tests {
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use zeloxy::{ProxyChain, ProxyResult};

  #[tokio::test]
  async fn test_mini_chain() -> ProxyResult<()> {
    let proxies = vec![
      "socks4://98.181.137.83:4145",
      "socks4://98.170.57.249:4145",
      "socks5://212.58.132.5:1080",
    ];

    let chain = ProxyChain::from(proxies);

    let mut stream = chain.connect("ipinfo.io", 80).await?;

    stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).await?;

    println!("{}", String::from_utf8_lossy(&resp));

    Ok(())
  }

  #[tokio::test]
  async fn test_chain_from() {
    let proxies = vec![
      "socks4://98.181.137.83:4145",
      "socks4://98.170.57.249:4145",
      "socks5://212.58.132.5:1080",
    ];

    println!("Цепочка: {:?}", ProxyChain::from(proxies));
  }
}
