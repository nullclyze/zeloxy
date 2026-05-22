# Установка и подключение библиотеки

Перед началом нужно подключить библиотеку `zeloxy`, добавив в `Cargo.toml` зависимость:

```toml
[dependencies]
zeloxy = { version = "0.2.2", features = ["all"] }
```

Или написать в терминале:

```bash
cargo add zeloxy
```

# Фичи библиотеки

## Доступные фичи

- `http`: поддеркжка для HTTP прокси.
- `socks4`: поддеркжка для SOCKS4 прокси.
- `socks5`: поддеркжка для SOCKS5 прокси.
- `chain`: функционал для работы с цепочками прокси.
- `stream`: функционал для создания прокси потоков.
- `tools`: вспомогательные инструменты, такие как `ping`, `lookup`.
- `all`: включает все фичи.

## Фичи по умолчанию

Набор фич по умолчанию включает в себя: `http`, `socks4`, `socks5`

# Первая программа

Давай для начала напишем простую программу, которая будет подключаться к `ipinfo.io:80` через SOCKS4 прокси. Здесь мы сможем точно понять, каким видят наш IP сторонние сайты, когда мы используем прокси.

```rust
use zeloxy::{Proxy, ProxyResult, ProxyStream, ProxyType};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём SOCKS4-прокси
  let proxy = Proxy::from("socks4://68.71.242.118:4145");

  // Создаём прокси поток
  let stream = ProxyStream::new(proxy);

  // Подключаемся к целевому серверу
  stream.connect("ipinfo.io", 80).await?;

  // Отправляем GET-запрос на ipinfo.io
  let buf = "GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n".as_bytes();
  stream.write(buf).await?;

  // Читаем ответ и логгируем его
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;
  println!("Ответ: {}", String::from_utf8_lossy(&resp));

  Ok(())
}
```

После запуска этой программы ты должен увидеть в терминале следующую информацию:

```
HTTP/1.0 200 OK
access-control-allow-origin: *
content-type: application/json
Content-Length: 265
date: Mon, 11 May 2026 16:24:21 GMT
vary: accept-encoding
via: 1.1 google

{
  "ip": "68.71.242.118", # Важно, здесь должен быть IP прокси
  "city": "Los Angeles",
  "region": "California",
  "country": "US",
  "loc": "34.0522,-118.2437",
  "org": "AS46562 Performive LLC",
  "postal": "90012",
  "timezone": "America/Los_Angeles",
  "readme": "https://ipinfo.io/missingauth"
}
```

# Цепочка прокси

Библиотека `zeloxy` имеет встроенный функционал для создания цепочек прокси.

## Что за цепочки

Всё просто - вместо одного промежуточного прокси мы создаём несколько узлов, это выглядит так:

```
Клиент (ты) <-> 1.0.0.0 <-> 2.0.0.0 <-> 3.0.0.0 <-> Целевой сервер
```

## Примечание

Цепочка прокси часто может работать очень медленно, ибо один промежуточный прокси замедляет передачу данных в X раз, а цепочка состоит из двух и более промежуточных прокси. Чем длинее цепочка - тем медленнее передача данных между клиентом и целевым сервером.

## Создание миксованной цепочки

Здесь мы создаздим в качестве примера миксованную цепочку прокси (цепочка, состоящия из прокси, работающих по разным протоколам). Для точного определения выходного IP мы вновь будем подключаться к `ipinfo.io:80`.

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{ProxyChain, ProxyResult};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём список прокси
  let proxies = vec![
    "socks4://98.181.137.83:4145",
    "socks4://98.170.57.249:4145",
    "socks5://212.58.132.5:1080", // Выходной прокси
  ];

  // Создаём цепочку прокси
  let mut chain = ProxyChain::from(proxies);

  // Подключаемся к целевому серверу через цепочку
  let mut stream = chain.connect("ipinfo.io", 80, false).await?;

  // Отправляем GET-запрос
  stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

  // Читаем ответ (в данном случае информация об IP)
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  // Логгируем выходной IP
  println!("{}", String::from_utf8_lossy(&resp));

  Ok(())
}
```

После запуска программы ты должен увидеть следующую информацию:

```
HTTP/1.0 200 OK
access-control-allow-origin: *
content-type: application/json
Content-Length: 308
date: Mon, 11 May 2026 16:45:24 GMT
vary: accept-encoding
via: 1.1 google

{
  "ip": "212.58.132.5", # Здесь должен быть выходной (последний) IP цепочки
  "hostname": "212-58-132-5.reverse.ip.nsfocus.cloud",
  "city": "Singapore",
  "region": "Singapore",
  "country": "SG",
  "loc": "1.2897,103.8501",
  "org": "AS8757 NSFOCUS, Inc.",
  "postal": "018989",
  "timezone": "Asia/Singapore",
  "readme": "https://ipinfo.io/missingauth"
}
```

# Использование инстументов

Библиотека `zeloxy` предлагает встроенные инструменты - пингование прокси, получение данных об IP. В этом разделе мы рассмотрим оба инструмента.

## Пингование прокси

Давай напишем программу, которая будет пинговать несколько прокси и выводить информацию в терминал.

```rust
use std::sync::Arc;

use zeloxy::Proxy;
use zeloxy::tools::ping_proxy_parallel;

#[tokio::main]
async fn main() {
  // Создаём список прокси
  let proxies = vec![
    "socks5://195.19.50.126:1080",
    "socks5://212.58.132.5:1080",
    "socks5://34.174.40.246:1080",
    "socks4://98.181.137.83:4145",
    "socks4://184.178.172.14:4145",
    "socks4://98.170.57.249:4145",
  ];

  let mut handles = Vec::new();

  for proxy_str in proxies {
    // Спавним отдельную задачу для более быстрой проверки
    let handle = tokio::spawn(async move {
      // Создаём прокси
      let proxy = Proxy::from(proxy_str);

      // Проверяем прокси, используя список целевых сервисов по умолчанию
      let result = ping_proxy_parallel(Arc::new(proxy), None).await;

      if result.pinged_services.len() > 0 {
        println!("\n============= {} =============", proxy_str);

        // Логгируем результаты
        for (name, ping) in result.pinged_services {
          println!("|- Пинг {}: {}ms", name, ping);
        }

        println!("|-------------------------------------------------------");

        if let Some(average_ping) = result.average_ping {
          println!("|- Средний пинг прокси: {}ms", average_ping);
        }

        println!("========================================================");
      } else {
        println!("\n[!] Прокси {} недоступен", proxy_str);
      }
    });

    handles.push(handle);
  }

  // Ожидаем завершения всех задач
  for handle in handles {
    let _ = handle.await;
  }
}
```

Вывод программы будет примерно таким:

```
============= socks4://98.181.137.83:4145 =============
|- Пинг cloudflare.com: 558ms
|- Пинг facebook.com: 638ms
|- Пинг yandex.ru: 693ms
|- Пинг github.com: 560ms
|- Пинг reddit.com: 557ms
|-------------------------------------------------------
|- Средний пинг прокси: 601ms
========================================================

============= socks5://212.58.132.5:1080 =============
|- Пинг reddit.com: 965ms
|- Пинг cloudflare.com: 1007ms
|- Пинг yandex.ru: 1162ms
|- Пинг github.com: 1224ms
|- Пинг facebook.com: 1251ms
|- Пинг youtube.com: 1017ms
|-------------------------------------------------------
|- Средний пинг прокси: 1104ms
========================================================

И так далее...
```

## Получения данных об IP

Здесь мы напишем программу, которая будет получать базовую информацию об IP.

```rust
use zeloxy::Proxy;
use zeloxy::tools::lookup_proxy;

#[tokio::main]
async fn main() {
  // Создаём SOCKS4-прокси
  let proxy = Proxy::from("socks4://98.181.137.83:4145");

  // Логгируем информацию об IP прокси
  if let Some(info) = lookup_proxy(&proxy).await {
    println!("Информация об IP: {:#?}", info);
  } else {
    println!("Не удалось получить информацию об IP");
  }
}
```

Вывод должен выглядеть так:

```
Информация об IP: LookupInfo {
  ip: "\"98.181.137.83\"",
  hostname: "null",
  city: "\"Las Vegas\"",
  region: "\"Nevada\"",
  country: "\"US\"",
  location: "\"36.1750,-115.1372\"",
  organization: "\"AS22773 Cox Communications Inc.\"",
  postal: "\"89111\"",
  timezone: "\"America/Los_Angeles\"",
}
```