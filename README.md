# Zeloxy

[Russian Documentation](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md) | [English Documentation](https://github.com/nullclyze/zeloxy/blob/main/docs/EN.md) | [Examples](https://github.com/nullclyze/zeloxy/blob/main/examples) | [License](https://github.com/nullclyze/zeloxy/blob/main/LICENSE)

A Rust library for working with various proxies.

Supported proxy types:

- **HTTP** (without auth / with basic auth)
- **SOCKS5** (without auth / with `user / pass` auth)
- **SOCKS4** (without auth / with `ident` auth)

# Focusing

- **Lightweight:** This crate does not use unnecessary dependencies and useless functionality.
- **Simplicity:** This crate contains clear and logical functionality with examples.
- **Built-in:** This crate offers built-in tools for convenient proxy management.
- **Asynchrony:** This crate is based on an asynchronous environment.
- **Lag-free:** This crate limits almost all operations with timeouts to prevent lags / freezes.

# Tasks and Goals

- [x] SOCKS4 proxy support
- [x] SOCKS5 proxy support
- [x] HTTP proxy support
- [ ] HTTPS proxy support
- [x] Basic authorizations support
- [x] Proxy chain implementation
- [ ] Automatic proxies reordering in chain on error
- [x] Built-in proxy stream
- [x] Proxy lookup
- [x] Proxy ping
- [ ] Proxy stealth check

# Examples

Current examples can be found here: [browse](https://github.com/nullclyze/zeloxy/tree/main/examples)

## Create a HTTP proxy stream

```rust
use zeloxy::{GetRequestOpts, Proxy, ProxyResult, ProxyStream, ProxyType};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём HTTP-прокси (в данном примере используется публичный прокси)
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http);

  // Создаём поток с прокси
  let stream = ProxyStream::new(proxy);

  // Подключаемся к целевому серверу
  stream.connect("example.com", 80).await?;

  // Отправляем GET-запрос на example.com
  let resp = stream.get_request("example.com", GetRequestOpts::default()).await?;

  // Логгируем ответ
  println!("Ответ от example.com: {}", resp);

  Ok(())
}
```

## Connect to SOCKS5 proxy

```rust
use zeloxy::{Proxy, ProxyResult};

#[tokio::main]
async fn main() {
  // Создаём SOCKS5-прокси (в данном примере используется публичный прокси)
  let proxy = Proxy::from("socks5://212.58.132.5:1080");

  // Подключаемся к прокси и логгируем результат
  match proxy.connect("ipinfo.io", 80).await {
    ProxyResult::Ok(_) => {
      println!("Подключение установлено");
    }
    ProxyResult::Err(e) => {
      println!("Ошибка подключения: {:?}", e);
    }
  }
}
```

## Create a proxy chain

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{Proxy, ProxyChain, ProxyResult};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём список прокси
  let proxies = vec![
    "socks4://98.181.137.83:4145",
    "socks4://98.170.57.249:4145",
    "socks5://212.58.132.5:1080",
  ];

  // Создаём цепочку прокси
  let chain = ProxyChain::from(proxies);

  // Подключаемся к целевому серверу через цепочку
  let mut stream = chain.connect("ipinfo.io", 80).await?;

  // Отправляем GET-запрос
  stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

  // Читаем ответ (в данном случае информация об IP)
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  // Логгируем выходной IP (здесь должно быть { "ip": "212.58.132.5:1080" })
  println!("Выход: {}", String::from_utf8_lossy(&resp));

  Ok(())
}
```
