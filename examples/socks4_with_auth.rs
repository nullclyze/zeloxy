use zeloxy::{Proxy, ProxyAuth, ProxyResult};

#[tokio::main]
async fn main() {
  // В реальном коде здесь должен быть валидный USER_ID.
  // Так же в SOCKS4 не указывается пароль, так как здесь
  // метод авторизации не требует его в отличии от SOCKS5 / HTTP
  let auth = ProxyAuth::new("USER_ID", "");

  // Создаём SOCKS4-прокси и задаём ему адрес целевого сервера.
  // В реальном коде должны быть валидные данные
  let proxy = Proxy::from("socks4://PROXY_IP:PROXY_PORT").with_auth(auth);

  // Подключаемся к прокси и логгируем результат
  match proxy.connect("TARGET_HOST", 80).await {
    ProxyResult::Ok(_) => {
      println!("Подключение с прокси установлено");
    }
    ProxyResult::Err(e) => {
      println!("Ошибка подключения: {:?}", e);
    }
  }
}
