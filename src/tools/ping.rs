use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

use hashbrown::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{Proxy, ProxyChain};

/// Структура информации о пинге прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingInfo {
  /// Пропингованные сервисы
  pub pinged_services: HashMap<String, u64>,

  /// Средний пинг прокси относительно пропингованных сервисов
  pub average_ping: Option<u64>,
}

/// Структура информации о пинге цепочки прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainPingInfo {
  /// Список промежуточного (from-to) пинга прокси
  pub from_to_ping_map: HashMap<(String, String), u64>,

  /// Общий from-to пинг
  pub from_to_total_ping: Option<u64>,

  /// Средний from-to пинг
  pub from_to_average_ping: Option<u64>,

  /// Пропингованные сервисы
  pub pinged_services: HashMap<String, u64>,

  /// Средний пинг прокси относительно пропингованных сервисов
  pub average_ping: Option<u64>,
}

/// Вспомогательная функция получения пингуемых сервисов по умолчанию
fn default_pinged_services() -> Vec<(String, u16)> {
  vec![
    ("cloudflare.com".to_string(), 80),
    ("facebook.com".to_string(), 80),
    ("yandex.ru".to_string(), 80),
    ("youtube.com".to_string(), 80),
    ("github.com".to_string(), 80),
    ("reddit.com".to_string(), 80),
  ]
}

/// Функция пингования прокси
pub async fn ping_proxy(proxy: &Proxy, pinged_services: Option<Vec<(String, u16)>>) -> PingInfo {
  let mut ping_info = PingInfo {
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  if !proxy.is_available().await {
    return ping_info;
  }

  let mut total_pinged_services = 0;
  let mut total_ping = 0;

  let ps = if let Some(services) = pinged_services {
    services
  } else {
    default_pinged_services()
  };

  for (service_host, service_port) in ps {
    let start_time = Instant::now();

    match proxy.connect(&service_host, service_port).await {
      Ok(mut s) => {
        let _ = s.shutdown().await;

        let ping = start_time.elapsed().as_millis() as u64;

        total_pinged_services += 1;
        total_ping += ping as u64;

        ping_info.pinged_services.insert(service_host, ping);
      }
      Err(_) => {}
    }
  }

  ping_info.average_ping = Some(total_ping / total_pinged_services as u64);

  ping_info
}

/// Функция пингования цепочки прокси.
/// Данная функция может требовать 11 и более секунд на
/// пинговку цепочки из 3 прокси, поэтому рекомендуется использовать
/// `ping_proxy_chain_parallel` для увеличения скорости пинговки
pub async fn ping_proxy_chain(chain: &ProxyChain, pinged_services: Option<Vec<(String, u16)>>) -> ChainPingInfo {
  let mut ping_info = ChainPingInfo {
    from_to_ping_map: HashMap::new(),
    from_to_total_ping: None,
    from_to_average_ping: None,
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  let mut total_pinged_services = 0;
  let mut total_ping = 0;

  let ps = if let Some(services) = pinged_services {
    services
  } else {
    default_pinged_services()
  };

  for (service_host, service_port) in ps {
    let proxy_chain = chain.get_chain();
    let first_proxy = &proxy_chain[0];
    let first_addr = first_proxy.get_address();

    let mut stream = match TcpStream::connect(first_addr).await {
      Ok(s) => s,
      Err(_) => continue,
    };

    let mut last_proxy_addr = first_addr;
    let mut from_to_total_ping = 0;
    let mut from_to_pinged = 0;

    for proxy in &proxy_chain[1..] {
      let proxy_ip = if let Some(ip) = proxy.get_ip() {
        ip
      } else {
        continue;
      };

      let proxy_port = if let Some(port) = proxy.get_port() {
        port
      } else {
        continue;
      };

      let start_time = Instant::now();

      stream = match proxy.connect_with_stream(stream, proxy_ip, proxy_port).await {
        Ok(s) => s,
        Err(_) => return ping_info, // Считается что цепочка поломана, поэтому кидаем что есть
      };

      let from_to = (last_proxy_addr.to_string(), proxy.get_address().to_string());
      let ping = start_time.elapsed().as_millis() as u64;

      ping_info.from_to_ping_map.insert(from_to, ping);

      last_proxy_addr = proxy.get_address();
      from_to_total_ping += ping;
      from_to_pinged += 1;
    }

    ping_info.from_to_total_ping = Some(from_to_total_ping);
    ping_info.from_to_average_ping = Some(from_to_total_ping / from_to_pinged);

    let last_proxy = &proxy_chain[proxy_chain.len() - 1];

    let start_time = Instant::now();

    match last_proxy.connect_with_stream(stream, &service_host, service_port).await {
      Ok(mut s) => {
        let _ = s.shutdown().await;

        let ping = start_time.elapsed().as_millis() as u64;

        total_pinged_services += 1;
        total_ping += ping;

        ping_info.pinged_services.insert(service_host, ping);
      }
      Err(_) => continue,
    }
  }

  let average_ping = total_ping / total_pinged_services;
  ping_info.average_ping = Some(average_ping);

  ping_info
}

/// Функция параллельного пингования прокси
pub async fn ping_proxy_parallel(proxy: Arc<Proxy>, pinged_services: Option<Vec<(String, u16)>>) -> PingInfo {
  let ping_info = PingInfo {
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  let ping_info_mutex = Arc::new(Mutex::new(ping_info));

  if !proxy.is_available().await {
    return ping_info_mutex.lock().await.clone();
  }

  let total_pinged_services = Arc::new(AtomicU8::new(0));
  let total_ping = Arc::new(AtomicU64::new(0));

  let mut handles = Vec::new();

  let ps = if let Some(services) = pinged_services {
    services
  } else {
    default_pinged_services()
  };

  for (service_host, service_port) in ps {
    let proxy_clone = Arc::clone(&proxy);
    let ping_info_mutex_clone = Arc::clone(&ping_info_mutex);
    let total_pinged_services_clone = Arc::clone(&total_pinged_services);
    let total_ping_clone = Arc::clone(&total_ping);

    let handle = tokio::spawn(async move {
      let start_time = Instant::now();

      match proxy_clone.connect(&service_host, service_port).await {
        Ok(mut s) => {
          let _ = s.shutdown().await;

          let ping = start_time.elapsed().as_millis() as u64;

          total_pinged_services_clone.fetch_add(1, Ordering::SeqCst);
          total_ping_clone.fetch_add(ping, Ordering::SeqCst);

          ping_info_mutex_clone.lock().await.pinged_services.insert(service_host, ping);
        }
        Err(_) => {}
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    let _ = handle.await;
  }

  let average_ping = total_ping.load(Ordering::SeqCst) / total_pinged_services.load(Ordering::SeqCst) as u64;
  ping_info_mutex.lock().await.average_ping = Some(average_ping);

  ping_info_mutex.lock().await.clone()
}

/// Функция параллельного пингования цепочки прокси
pub async fn ping_proxy_chain_parallel(chain: Arc<ProxyChain>, pinged_services: Option<Vec<(String, u16)>>) -> ChainPingInfo {
  let ping_info = ChainPingInfo {
    from_to_ping_map: HashMap::new(),
    from_to_total_ping: None,
    from_to_average_ping: None,
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  let ping_info_mutex = Arc::new(Mutex::new(ping_info));

  let total_pinged_services = Arc::new(AtomicU8::new(0));
  let total_ping = Arc::new(AtomicU64::new(0));

  let mut handles = Vec::new();

  let ps = if let Some(services) = pinged_services {
    services
  } else {
    default_pinged_services()
  };

  for (service_host, service_port) in ps {
    let chain_clone = Arc::clone(&chain);
    let ping_info_mutex_clone = Arc::clone(&ping_info_mutex);
    let total_pinged_services_clone = Arc::clone(&total_pinged_services);
    let total_ping_clone = Arc::clone(&total_ping);

    let handle = tokio::spawn(async move {
      let proxy_chain = chain_clone.get_chain();
      let first_proxy = &proxy_chain[0];
      let first_addr = first_proxy.get_address();

      let mut stream = match TcpStream::connect(first_addr).await {
        Ok(s) => s,
        Err(_) => return,
      };

      let mut last_proxy_addr = first_addr;
      let mut from_to_total_ping = 0;
      let mut from_to_pinged = 0;

      for proxy in &proxy_chain[1..] {
        let proxy_ip = if let Some(ip) = proxy.get_ip() {
          ip
        } else {
          return;
        };

        let proxy_port = if let Some(port) = proxy.get_port() {
          port
        } else {
          return;
        };

        let start_time = Instant::now();

        stream = match proxy.connect_with_stream(stream, proxy_ip, proxy_port).await {
          Ok(s) => s,
          Err(_) => return,
        };

        let from_to = (last_proxy_addr.to_string(), proxy.get_address().to_string());
        let ping = start_time.elapsed().as_millis() as u64;

        ping_info_mutex_clone.lock().await.from_to_ping_map.insert(from_to, ping);

        last_proxy_addr = proxy.get_address();
        from_to_total_ping += ping;
        from_to_pinged += 1;
      }

      ping_info_mutex_clone.lock().await.from_to_total_ping = Some(from_to_total_ping);
      ping_info_mutex_clone.lock().await.from_to_average_ping = Some(from_to_total_ping / from_to_pinged);

      let last_proxy = &proxy_chain[proxy_chain.len() - 1];

      let start_time = Instant::now();

      match last_proxy.connect_with_stream(stream, &service_host, service_port).await {
        Ok(mut s) => {
          let _ = s.shutdown().await;

          let ping = start_time.elapsed().as_millis();

          total_pinged_services_clone.fetch_add(1, Ordering::SeqCst);
          total_ping_clone.fetch_add(ping as u64, Ordering::SeqCst);

          ping_info_mutex_clone.lock().await.pinged_services.insert(service_host, ping as u64);
        }
        Err(_) => return,
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    let _ = handle.await;
  }

  let average_ping = total_ping.load(Ordering::SeqCst) / total_pinged_services.load(Ordering::SeqCst) as u64;
  ping_info_mutex.lock().await.average_ping = Some(average_ping);

  ping_info_mutex.lock().await.clone()
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::tools::{ping_proxy, ping_proxy_chain, ping_proxy_chain_parallel, ping_proxy_parallel};
  use crate::{Proxy, ProxyChain, ProxyType};

  #[tokio::test]
  async fn test_ping_proxy() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    let result = ping_proxy(&proxy, None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг прокси: {}ms", average_ping);
    }
  }

  #[tokio::test]
  async fn test_ping_proxy_parallel() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    let result = ping_proxy_parallel(Arc::new(proxy), None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг прокси: {}ms", average_ping);
    }
  }

  #[tokio::test]
  async fn test_ping_proxy_chain() {
    let proxies = vec![
      Proxy::from("socks4://98.181.137.83:4145"),
      Proxy::from("socks4://98.170.57.249:4145"),
      Proxy::from("socks5://212.58.132.5:1080"),
    ];

    let chain = ProxyChain::new().with_proxies(proxies);

    let result = ping_proxy_chain(&chain, None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    for ((from, to), ping) in result.from_to_ping_map {
      println!("Пинг {} <-> {}: {}ms", from, to, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг цепочки прокси: {}ms", average_ping);
    }

    if let Some(from_to_average_ping) = result.from_to_average_ping {
      println!("Средний from-to пинг цепочки прокси: {}ms", from_to_average_ping);
    }

    if let Some(from_to_total_ping) = result.from_to_total_ping {
      println!("Общий from-to пинг цепочки прокси: {}ms", from_to_total_ping);
    }
  }

  #[tokio::test]
  async fn test_ping_proxy_chain_parallel() {
    let proxies = vec![
      Proxy::from("socks4://98.181.137.83:4145"),
      Proxy::from("socks4://98.170.57.249:4145"),
      Proxy::from("socks5://212.58.132.5:1080"),
    ];

    let chain = ProxyChain::new().with_proxies(proxies);

    let result = ping_proxy_chain_parallel(Arc::new(chain), None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    for ((from, to), ping) in result.from_to_ping_map {
      println!("Пинг {} <-> {}: {}ms", from, to, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг цепочки прокси: {}ms", average_ping);
    }

    if let Some(from_to_average_ping) = result.from_to_average_ping {
      println!("Средний from-to пинг цепочки прокси: {}ms", from_to_average_ping);
    }

    if let Some(from_to_total_ping) = result.from_to_total_ping {
      println!("Общий from-to пинг цепочки прокси: {}ms", from_to_total_ping);
    }
  }
}
