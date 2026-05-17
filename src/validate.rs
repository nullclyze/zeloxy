/// Функция валидации строковой записи прокси.
///
/// ## Примеры результатов
///
/// - `http://77.14.58.23:80` -> `true`
/// - `http://77.14.58.23` -> `false` (не указан порт)
/// - `socks5://77.14.58.23:1080` -> `true`
/// - `socks5://77.14.58:1080` -> `false` (неверный формат IP-адреса)
/// - `socks4://user:pass@77.14.58.23:4145` -> `false` (в SOCKS4 указывается только USER_ID)
/// - `socks5://user:pass@77.14.58.23:1080` -> `true`
/// - `socks4://user_id@77.14.58.23:4145` -> `true`
pub fn validate_proxy_str(proxy: impl Into<String>) -> bool {
  let proxy_str = proxy.into();
  let proxy_split = proxy_str.split("://").collect::<Vec<&str>>();

  if proxy_split.len() != 2 {
    return false;
  }

  let without_protocol = proxy_split[1].split("@").collect::<Vec<&str>>();

  if without_protocol.len() > 2 {
    return false;
  }

  let (protocol, auth, addr) = {
    if without_protocol.len() == 2 {
      (proxy_split[0], Some(without_protocol[0]), without_protocol[1])
    } else {
      (proxy_split[0], None, without_protocol[0])
    }
  };

  if !["http", "socks4", "socks5"].contains(&protocol) {
    return false;
  }

  if let Some(a) = auth {
    let auth_data_count = a.split(":").count();

    if protocol == "socks4" {
      if auth_data_count != 1 {
        return false;
      }
    } else {
      if auth_data_count != 2 {
        return false;
      }
    }
  }

  let addr_split = addr.split(":").collect::<Vec<&str>>();

  if addr_split.len() != 2 {
    return false;
  }

  let ip_octecs_count = addr_split[0].split(".").count();

  if ip_octecs_count != 4 {
    return false;
  }

  true
}
