# `zeloxy`

A Rust library for working with various proxies.

Supported proxy types:

- **HTTP** (without auth / with basic auth)
- **SOCKS5** (without auth / with `user / pass` auth)
- **SOCKS4** (without auth / with `ident` auth)

# Examples

Current examples can be found here: [browse](https://github.com/nullclyze/zeloxy/tree/main/examples)

## Connect to SOCKS5 proxy

```rust
use zeloxy::{Proxy, ProxyResult};

#[tokio::main]
async fn main() {
  // Создаём SOCKS5-прокси и задаём ему адрес целевого сервера.
  // В данном примере используется публичный прокси
  let proxy = Proxy::from("socks5://212.58.132.5:1080").bind("ipinfo.io", 80);

  // Подключаемся к прокси и логгируем результат
  match proxy.connect().await {
    ProxyResult::Ok(_) => {
      println!("Подключение с прокси установлено");
    }
    ProxyResult::Err(e) => {
      println!("Ошибка подключения: {:?}", e);
    }
  }
}
```