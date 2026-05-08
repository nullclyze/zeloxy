use std::time::Instant;

use hashbrown::HashMap;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{Proxy, ProxyResult};

/// Структура информации об IP
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IpInfo {
  pub ip: String,
  pub hostname: String,
  pub city: String,
  pub region: String,
  pub country: String,
  pub loc: String,
  pub org: String,
  pub postal: String,
  pub timezone: String,
  pub readme: String,
}

/// Структура результата проверки
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
  pub pinged_services: HashMap<String, u128>,
  pub average_ping: Option<u128>,
}

/// Трейт прокси чекера
pub trait ProxyChecker {
  /// Метод проверки качества прокси, основываясь на результатах подключения
  /// к различным популярным сервисам вроде Cloudflare, ChatGPT и так далее.
  /// Важно понимать, что данная проверка может занимать определённое время,
  /// обычно требуется ~7 секунд чтобы проверить доступность всех сервисов.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use zeloxy::{Proxy, ProxyChecker};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   // Создаём прокси
  ///   let proxy = Proxy::new("PROXY_IP:PROXY_PORT");
  ///
  ///   // Проверяем доступность прокси
  ///   let check_result = proxy.check_proxy().await;
  ///
  ///   // Логгируем результат проверки
  ///   for (name, ping) in result.pinged_services {
  ///     println!("Пинг {}: {}ms", name, ping);
  ///   }
  ///
  ///   println!("===============================");
  ///
  ///   if let Some(average_ping) = result.average_ping {
  ///     println!("Средний пинг прокси: {}ms", average_ping);
  ///   }
  /// }
  /// ```
  fn check_proxy(&self) -> impl std::future::Future<Output = CheckResult> + Send;

  /// Метод получения информации об IP с `ipinfo.io`.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use zeloxy::{Proxy, ProxyChecker};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   // Создаём прокси и получаем информацию об IP
  ///   let proxy = Proxy::new("PROXY_IP:PROXY_PORT");
  ///   let ip_info = proxy.get_ip_info().await;
  ///
  ///   println!("Имя хоста: {}", ip_info.hostname);
  ///   println!("Страна: {}", ip_info.country);
  ///   println!("Город: {}", ip_info.city);
  ///   println!("Локация: {}", ip_info.loc);
  /// }
  /// ```
  fn lookup(&self) -> impl std::future::Future<Output = Option<IpInfo>> + Send;
}

impl ProxyChecker for Proxy {
  async fn check_proxy(&self) -> CheckResult {
    let mut check_result = CheckResult {
      pinged_services: HashMap::new(),
      average_ping: None,
    };

    if !self.is_available().await {
      return check_result;
    }

    let services = vec![
      ("cloudflare.com", 80),
      ("chatgpt.com", 80),
      ("facebook.com", 80),
      ("yandex.ru", 80),
      ("youtube.com", 80),
      ("github.com", 80),
      ("reddit.com", 80),
    ];

    let mut total_pinged_services = 0;
    let mut total_ping = 0;

    for (service_host, service_port) in services {
      if let Some(ping) = ping_service(&self, service_host, service_port).await {
        total_pinged_services += 1;
        total_ping += ping;

        check_result.pinged_services.insert(service_host.to_string(), ping);
      }
    }

    check_result.average_ping = Some(total_ping / total_pinged_services);

    check_result
  }

  async fn lookup(&self) -> Option<IpInfo> {
    self.rebind("ipinfo.io".to_string(), 80);

    let mut stream = match self.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(_) => return None,
    };

    let _ = stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await;

    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await;

    let data = match String::from_utf8(buf) {
      Ok(s) => s,
      Err(_) => String::new(),
    };

    let split_data = data.split("\n").collect::<Vec<&str>>();
    let mut pretty_data = String::new();

    // Просто скипаем заголовки
    for (i, item) in split_data.iter().enumerate() {
      if i < 7 {
        continue;
      }

      pretty_data.push_str(*item);
    }

    let ip_info: IpInfo = match serde_json::from_str(&pretty_data) {
      Ok(info) => info,
      Err(_) => return None,
    };

    Some(ip_info)
  }
}

/// Вспомогательная функция пингования сервиса
async fn ping_service(proxy: &Proxy, service_host: &str, service_port: u16) -> Option<u128> {
  proxy.rebind(service_host, service_port);

  let start_time = Instant::now();

  match proxy.connect().await {
    Ok(_) => {}
    Err(_) => return None,
  }

  Some(start_time.elapsed().as_millis())
}

#[cfg(test)]
mod tests {
  use crate::{Proxy, ProxyChecker, ProxyType};

  #[tokio::test]
  async fn test_proxy_check() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    let result = proxy.check_proxy().await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг прокси: {}ms", average_ping);
    }
  }

  #[tokio::test]
  async fn test_lookup() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);
    println!("Информация об IP: {:?}", proxy.lookup().await);
  }
}
