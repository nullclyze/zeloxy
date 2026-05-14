use zeloxy::{Proxy, ProxyResult, ProxyStream, ProxyType};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём HTTP-прокси
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http);

  // Создаём поток с прокси
  let stream = ProxyStream::new(proxy);

  // Подключаемся к целевому серверу
  stream.connect("example.com", 80).await?;

  // Отправляем GET-запрос на example.com
  let buf = "GET / HTTP/1.0\r\nHost: example.com\r\n\r\n".as_bytes();
  stream.write(buf).await?;

  // Читаем ответ и логгируем его
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  println!("Ответ: {}", String::from_utf8_lossy(&resp));

  Ok(())
}
