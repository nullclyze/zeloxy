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
    "http://45.131.6.46:80",
    "http://91.98.78.64:80",
    "http://63.141.128.27:80",
  ];

  let mut handles = Vec::new();

  for proxy_str in proxies {
    // Спавним отдельную задачу для более быстрой проверки
    let handle = tokio::spawn(async move {
      // Создаём прокси
      let proxy = Proxy::from(proxy_str);

      // Проверяем прокси
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
