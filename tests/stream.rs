use zeloxy::{GetRequestOpts, Proxy, ProxyResult, ProxyStream};

#[tokio::test]
async fn test_proxy_stream() -> ProxyResult<()> {
  let proxy = Proxy::from("socks4://68.71.242.118:4145");
  let stream = ProxyStream::new(proxy);

  stream.connect("ipinfo.io", 80).await?;

  let options = GetRequestOpts {
    user_agent: Some(
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
    ),
    ..Default::default()
  };

  println!("{}", options.to_request());

  let resp = stream.get_request("ipinfo.io", options).await?;

  println!("Ответ: {}", resp);

  Ok(())
}
