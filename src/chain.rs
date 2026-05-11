use tokio::net::TcpStream;

use crate::{ErrorKind, Proxy, ProxyError, ProxyResult};

/// Цепочка прокси.
///
/// ## Примеры
///
/// ```rust, ignore
/// use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// use zeloxy::{Proxy, ProxyChain, ProxyResult};
///
/// #[tokio::main]
/// async fn main() -> ProxyResult<()> {
///   // Создаём список из 3 прокси
///   let proxies = vec![
///     "socks4://98.181.137.83:4145",
///     "socks4://98.170.57.249:4145",
///     "socks5://212.58.132.5:1080",
///   ];
///
///   // Создаём цепочку прокси
///   let chain = ProxyChain::from(proxies);
///
///   // Подключаемся к целевому серверу через цепочку
///   let mut stream = chain.connect("ipinfo.io", 80).await?;
///
///   // Отправляем GET-запрос
///   stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;
///
///   // Читаем ответ (в данном случае информация об IP)
///   let mut resp = Vec::new();
///   stream.read_to_end(&mut resp).await?;
///
///   // Логгируем выходной IP (здесь должно быть { "ip": "212.58.132.5:1080" })
///   println!("Выход: {}", String::from_utf8_lossy(&resp));
///
///   Ok(())
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/nullclyze/zeloxy/tree/main/examples)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyChain {
  chain: Vec<Proxy>,
}

impl ProxyChain {
  /// Метод создания пустой цепочки прокси
  pub fn new() -> Self {
    Self { chain: Vec::new() }
  }

  /// Метод добавления прокси в цепочку
  pub fn add_proxy(&mut self, proxy: impl Into<Proxy>) {
    self.chain.push(proxy.into());
  }

  /// Метод добавления нескольких прокси в цепочку
  pub fn add_proxies(&mut self, proxies: Vec<Proxy>) {
    self.chain.extend(proxies);
  }

  /// Метод добавления прокси в цепочку (возвращает `Self`)
  pub fn with_proxy(mut self, proxy: impl Into<Proxy>) -> Self {
    self.chain.push(proxy.into());
    self
  }

  /// Метод добавления нескольких прокси в цепочку (возвращает `Self`)
  pub fn with_proxies(mut self, proxies: Vec<Proxy>) -> Self {
    self.chain.extend(proxies);
    self
  }

  /// Метод очистки цепочки прокси
  pub fn clear(&mut self) {
    self.chain.clear();
  }

  /// Метод получения цепочки прокси
  pub fn get_chain(&self) -> &Vec<Proxy> {
    &self.chain
  }

  /// Метод подключения к целевому серверу через цепочку прокси
  pub async fn connect(&self, target_host: impl Into<String>, target_port: u16) -> ProxyResult<TcpStream> {
    let first_proxy = &self.chain[0];
    let first_addr = first_proxy.get_address();

    let mut stream = TcpStream::connect(first_addr).await?;

    for proxy in &self.chain[1..] {
      let proxy_ip = proxy
        .get_ip()
        .ok_or(ProxyError::new(ErrorKind::InvalidData, "failed to obtain proxy IP address"))?;

      let proxy_port = proxy
        .get_port()
        .ok_or(ProxyError::new(ErrorKind::InvalidData, "failed to obtain proxy port"))?;

      stream = proxy.connect_with_stream(stream, proxy_ip, proxy_port).await?;
    }

    let last_proxy = &self.chain[self.chain.len() - 1];

    stream = last_proxy.connect_with_stream(stream, target_host, target_port).await?;

    Ok(stream)
  }
}

impl From<Vec<String>> for ProxyChain {
  fn from(value: Vec<String>) -> Self {
    let mut chain = Vec::new();

    for address in value {
      chain.push(Proxy::from(address));
    }

    Self { chain }
  }
}

impl From<Vec<&str>> for ProxyChain {
  fn from(value: Vec<&str>) -> Self {
    let mut chain = Vec::new();

    for address in value {
      chain.push(Proxy::from(address));
    }

    Self { chain }
  }
}

impl From<&str> for ProxyChain {
  fn from(value: &str) -> Self {
    let pretty_value = value.replace(" ", "").replace("\n", "");

    let chain: Vec<&str> = pretty_value.split(",").collect();

    Self::from(chain)
  }
}

impl From<String> for ProxyChain {
  fn from(value: String) -> Self {
    let pretty_value = value.replace(" ", "").replace("\n", "");

    let chain: Vec<&str> = pretty_value.split(",").collect();

    Self::from(chain)
  }
}

#[cfg(test)]
mod tests {
  use tokio::io::{AsyncReadExt, AsyncWriteExt};

  use crate::{Proxy, ProxyChain, ProxyResult};

  #[tokio::test]
  async fn test_mini_chain() -> ProxyResult<()> {
    let proxies = vec![
      Proxy::from("socks4://98.181.137.83:4145"),
      Proxy::from("socks4://98.170.57.249:4145"),
      Proxy::from("socks5://212.58.132.5:1080"),
    ];

    let chain = ProxyChain::new().with_proxies(proxies);

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
