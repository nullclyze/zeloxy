use std::time::Duration;

use rand::seq::SliceRandom;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::{Proxy, ProxyResult};

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
///   let mut chain = ProxyChain::from(proxies);
///
///   // Подключаемся к целевому серверу через цепочку
///   let mut stream = chain.connect("ipinfo.io", 80, false).await?;
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
  pub fn add_proxies(&mut self, proxies: Vec<impl Into<Proxy>>) {
    let mut iter = Vec::new();

    for proxy in proxies {
      iter.push(proxy.into());
    }

    self.chain.extend(iter);
  }

  /// Метод добавления прокси в цепочку (возвращает `Self`)
  pub fn with_proxy(mut self, proxy: impl Into<Proxy>) -> Self {
    self.chain.push(proxy.into());
    self
  }

  /// Метод добавления нескольких прокси в цепочку (возвращает `Self`)
  pub fn with_proxies(mut self, proxies: Vec<impl Into<Proxy>>) -> Self {
    let mut iter = Vec::new();

    for proxy in proxies {
      iter.push(proxy.into());
    }

    self.chain.extend(iter);
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

  /// Метод подключения к целевому серверу через цепочку прокси.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use zeloxy::ProxyChain;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   // Создаём список прокси
  ///   let proxies = vec![
  ///     "socks4://98.181.137.83:4145",
  ///     "socks4://98.170.57.249:4145",
  ///     "socks5://212.58.132.5:1080",
  ///   ];
  ///
  ///   // Создаём цепочку
  ///   let mut chain = ProxyChain::from(proxies);
  ///
  ///   // Подключаемся к целевому серверу и логгируем результат
  ///   match chain.connect("example.com", 80, false).await {
  ///     Ok(_) => println!("Подключение установлено"),
  ///     Err(_) => println!("Не удалось подключиться к серверу"),
  ///   }
  /// }
  /// ```
  ///
  /// Больше актуальных примеров: [смотреть](https://github.com/nullclyze/zeloxy/tree/main/examples)
  pub async fn connect(&mut self, target_host: impl Into<String>, target_port: u16, autoreorder: bool) -> ProxyResult<TcpStream> {
    let trgt_host = target_host.into();
    let trgt_port = target_port;

    if autoreorder {
      loop {
        let chain_clone = self.chain.clone();

        let first_proxy = &chain_clone[0];
        let first_addr = first_proxy.get_address();

        let mut stream = match tokio::time::timeout(Duration::from_secs(15), TcpStream::connect(first_addr)).await {
          Ok(r) => match r {
            Ok(s) => s,
            Err(_) => {
              self.reorder();
              continue;
            }
          },
          Err(_) => {
            self.reorder();
            continue;
          }
        };

        let mut chain_is_broken = false;

        for proxy in &chain_clone[1..] {
          let proxy_ip = proxy.get_ip();
          let proxy_port = proxy.get_port();

          match proxy.connect_with_stream(&mut stream, proxy_ip, proxy_port).await {
            Ok(_) => {}
            Err(_) => {
              let _ = stream.shutdown().await;
              self.reorder();
              chain_is_broken = true;
              break;
            }
          };
        }

        if chain_is_broken {
          continue;
        }

        let last_proxy = &chain_clone[chain_clone.len() - 1];

        match last_proxy.connect_with_stream(&mut stream, &trgt_host, trgt_port).await {
          Ok(_) => {}
          Err(_) => {
            let _ = stream.shutdown().await;
            self.reorder();
            continue;
          }
        };

        break Ok(stream);
      }
    } else {
      let first_proxy = &self.chain[0];
      let first_addr = first_proxy.get_address();

      let mut stream = TcpStream::connect(first_addr).await?;

      for proxy in &self.chain[1..] {
        let proxy_ip = proxy.get_ip();
        let proxy_port = proxy.get_port();

        proxy.connect_with_stream(&mut stream, proxy_ip, proxy_port).await?;
      }

      let last_proxy = &self.chain[self.chain.len() - 1];
      last_proxy.connect_with_stream(&mut stream, &trgt_host, trgt_port).await?;

      Ok(stream)
    }
  }

  /// Метод переупорядочивания цепочки прокси
  pub fn reorder(&mut self) {
    let mut rng = rand::rng();
    self.chain.shuffle(&mut rng);
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
