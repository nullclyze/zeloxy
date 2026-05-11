use std::sync::Arc;

use zeloxy::tools::ping_proxy_chain_parallel;
use zeloxy::{Proxy, ProxyChain};

#[tokio::main]
async fn main() {
  // Создаём список прокси
  let proxies = vec![
    Proxy::from("socks4://98.181.137.83:4145"),
    Proxy::from("socks4://98.170.57.249:4145"),
    Proxy::from("socks5://212.58.132.5:1080"),
  ];

  // Создаём цепочку прокси
  let chain = ProxyChain::new().with_proxies(proxies);

  // Пингуем цепочку прокси параллельно
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
