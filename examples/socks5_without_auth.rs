use zeloxy::{Proxy, ProxyResult};

#[tokio::main]
async fn main() {
  // Создаём SOCKS5-прокси (в данном примере используется публичный прокси)
  let proxy = Proxy::from("socks5://212.58.132.5:1080");

  // Подключаемся к прокси и логгируем результат
  match proxy.connect("ipinfo.io", 80).await {
    ProxyResult::Ok(_) => {
      println!("Подключение с прокси установлено");
    }
    ProxyResult::Err(e) => {
      println!("Ошибка подключения: {:?}", e);
    }
  }
}
