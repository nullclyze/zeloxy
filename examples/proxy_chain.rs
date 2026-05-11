use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{Proxy, ProxyChain, ProxyResult};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Создаём список из 3 прокси
  let proxies = vec![
    Proxy::from("socks4://98.181.137.83:4145"),
    Proxy::from("socks4://98.170.57.249:4145"),
    Proxy::from("socks5://212.58.132.5:1080"),
  ];

  // Создаём цепочку прокси
  let chain = ProxyChain::new().with_proxies(proxies);

  // Подключаемся к целевому серверу через цепочку
  let mut stream = chain.connect("ipinfo.io", 80).await?;

  // Отправляем GET-запрос
  stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

  // Читаем ответ (в данном случае информация об IP)
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  // Логгируем выходной IP (здесь должно быть { "ip": "212.58.132.5:1080" })
  println!("Выход: {}", String::from_utf8_lossy(&resp));

  Ok(())
}
