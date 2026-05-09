# `zeloxy`

A Rust library for working with various proxies.

Supported proxy types:

- **HTTP** (without auth / with basic auth)
- **SOCKS5** (without auth / with `user / pass` auth)
- **SOCKS4** (without auth / with `ident` auth)

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