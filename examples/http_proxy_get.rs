use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{Proxy, ProxyResult, ProxyType};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём HTTP-прокси
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http);

  match proxy.connect("ipinfo.io", 80).await {
    ProxyResult::Ok(mut conn) => {
      // Отправляем GET-запрос
      conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

      // Читаем ответ
      let mut resp = Vec::new();
      conn.read_to_end(&mut resp).await?;

      // Логгируем ответ
      println!("{}", String::from_utf8_lossy(&resp));
    }
    ProxyResult::Err(_) => {} // Просто игнорируем ошибки
  }

  Ok(())
}
