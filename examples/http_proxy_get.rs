use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{Proxy, ProxyResult, ProxyType};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём HTTP-прокси и задаём адрес целевого сервера
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http).bind("ipinfo.io".to_string(), 80);

  match proxy.connect().await {
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
