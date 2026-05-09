use zeloxy::{GetRequestOpts, Proxy, ProxyResult, ProxyStream, ProxyType};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём HTTP-прокси
  let proxy = Proxy::new("91.132.92.231:80", ProxyType::Http);

  // Создаём поток с прокси
  let stream = ProxyStream::new(proxy);

  // Подключаемся к целевому серверу
  stream.connect("example.com", 80).await?;

  // Отправляем GET-запрос на example.com
  let resp = stream.get_request("example.com", GetRequestOpts::default()).await?;

  // Логгируем ответ
  println!("Ответ от example.com: {}", resp);

  Ok(())
}
