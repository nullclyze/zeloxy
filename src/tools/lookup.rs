use serde::Deserialize;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::Proxy;

/// Структура информации об IP
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LookupInfo {
  pub ip: String,
  pub hostname: String,
  pub city: String,
  pub region: String,
  pub country: String,
  pub location: String,
  pub organization: String,
  pub postal: String,
  pub timezone: String,
}

/// Функция получения данных об IP адресе, используя `ipinfo.io`
pub async fn lookup_proxy(proxy: &Proxy) -> Option<LookupInfo> {
  let mut stream = match proxy.connect("ipinfo.io", 80).await {
    Ok(s) => s,
    Err(_) => return None,
  };

  let _ = stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await;

  let mut buf = Vec::new();
  let _ = stream.read_to_end(&mut buf).await;

  let data = match String::from_utf8(buf) {
    Ok(s) => s,
    Err(_) => String::new(),
  };

  let split_data = data.split("\n").collect::<Vec<&str>>();
  let mut pretty_data = String::new();

  for (i, item) in split_data.iter().enumerate() {
    if i < 7 {
      continue;
    }

    pretty_data.push_str(*item);
  }

  let raw: Value = match serde_json::from_str(&pretty_data) {
    Ok(info) => info,
    Err(_) => return None,
  };

  Some(LookupInfo {
    ip: raw["ip"].to_string(),
    hostname: raw["hostname"].to_string(),
    city: raw["city"].to_string(),
    region: raw["region"].to_string(),
    country: raw["country"].to_string(),
    location: raw["loc"].to_string(),
    organization: raw["org"].to_string(),
    postal: raw["postal"].to_string(),
    timezone: raw["timezone"].to_string(),
  })
}
