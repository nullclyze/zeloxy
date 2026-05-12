use zeloxy::Proxy;
use zeloxy::tools::lookup_proxy;

#[tokio::test]
async fn test_lookup() {
  let proxy = Proxy::from("socks4://98.181.137.83:4145");

  if let Some(info) = lookup_proxy(&proxy).await {
    println!("Информация об IP: {:#?}", info);
  } else {
    println!("Не удалось получить информацию об IP");
  }
}
