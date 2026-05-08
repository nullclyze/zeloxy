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
