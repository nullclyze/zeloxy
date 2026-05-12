#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use zeloxy::tools::{ping_proxy, ping_proxy_chain, ping_proxy_chain_parallel, ping_proxy_parallel};
  use zeloxy::{Proxy, ProxyChain, ProxyType};

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
      "socks4://98.181.137.83:4145",
      "socks4://98.170.57.249:4145",
      "socks5://212.58.132.5:1080",
    ];

    let chain = ProxyChain::from(proxies);

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
      "socks4://98.181.137.83:4145",
      "socks4://98.170.57.249:4145",
      "socks5://212.58.132.5:1080",
    ];

    let chain = ProxyChain::from(proxies);

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
