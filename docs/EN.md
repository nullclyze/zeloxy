# Installing and include library

Before we begin, we need to connect the `zeloxy` library by adding following dependency to `Cargo.toml`:

```toml
[dependencies]
zeloxy = "0.2.1" # A different version may be needed
```

Or type in terminal:

```bash
cargo add zeloxy
```

# First program

Let's start by writing a simple program that will connect to `ipinfo.io:80` through a SOCKS4 proxy. This will allow us to accurately understand how third-party sites see our IP address when we use a proxy.

```rust
use zeloxy::{GetRequestOpts, Proxy, ProxyResult, ProxyStream, ProxyType};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Create a SOCKS4 proxy
  let proxy = Proxy::from("socks4://68.71.242.118:4145");

  // Create a stream with the proxy
  let stream = ProxyStream::new(proxy);

  // Connect to the target server
  stream.connect("ipinfo.io", 80).await?;

  // Send a GET request to ipinfo.io
  let resp = stream.get_request("ipinfo.io", GetRequestOpts::default()).await?;

  // Log the response
  println!("Response: {}", resp);

  Ok(())
}
```

After running this program, you should see following information in terminal:

```
HTTP/1.0 200 OK
access-control-allow-origin: *
content-type: application/json
Content-Length: 265
date: Mon, 11 May 2026 16:24:21 GMT
vary: accept-encoding
via: 1.1 google

{
  "ip": "68.71.242.118", # Important, this must be proxy IP
  "city": "Los Angeles",
  "region": "California",
  "country": "US",
  "loc": "34.0522,-118.2437",
  "org": "AS46562 Performive LLC",
  "postal": "90012",
  "timezone": "America/Los_Angeles",
  "readme": "https://ipinfo.io/missingauth"
}
```

# Proxy Chain

The `zeloxy` library has built-in functionality for creating proxy chains.

## What are chains

It's simple - instead of a single intermediate proxy, we create multiple nodes, which looks like this:

```
Client (you) <-> 1.0.0.0 <-> 2.0.0.0 <-> 3.0.0.0 <-> Destination Server
```

## Note

A proxy chain can often be very slow, as a single intermediate proxy slows down data transfer by a factor of X, while a chain consists of two or more intermediate proxies. The longer the chain, the slower the data transfer between the client and the destination server.

## Creating a mixed chain

Here we'll create a mixed proxy chain as an example (a chain consisting of proxies operating on different protocols). To accurately determine the output IP, we'll again connect to `ipinfo.io:80`.

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeloxy::{ProxyChain, ProxyResult};

#[tokio::main]
async fn main() -> ProxyResult<()> {
  // Create a list of proxies
  let proxies = vec![
    "socks4://98.181.137.83:4145",
    "socks4://98.170.57.249:4145",
    "socks5://212.58.132.5:1080", // Exit IP
  ];

  // Create a proxy chain
  let chain = ProxyChain::from(proxies);

  // Connect to the target server via the chain
  let mut stream = chain.connect("ipinfo.io", 80).await?;

  // Send a GET request
  stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

  // Read the response (in this case, IP information)
  let mut resp = Vec::new();
  stream.read_to_end(&mut resp).await?;

  // Log the exit IP
  println!("{}", String::from_utf8_lossy(&resp));

  Ok(())
}
```

After running the program, you should see the following information:

```
HTTP/1.0 200 OK
access-control-allow-origin: *
content-type: application/json
Content-Length: 308
date: Mon, 11 May 2026 16:45:24 GMT
vary: accept-encoding
via: 1.1 google

{
  "ip": "212.58.132.5", # This should be the exit (last) IP of the chain
  "hostname": "212-58-132-5.reverse.ip.nsfocus.cloud",
  "city": "Singapore",
  "region": "Singapore",
  "country": "SG",
  "loc": "1.2897,103.8501",
  "org": "AS8757 NSFOCUS, Inc.",
  "postal": "018989",
  "timezone": "Asia/Singapore",
  "readme": "https://ipinfo.io/missingauth"
}
```

# Using tools

The `zeloxy` library offers built-in tools for pinging proxies and proxy lookup. In this section, we'll cover both tools.

## Pinging proxies

Let's write a program that will ping multiple proxies and output the information to the terminal.

```rust
use std::sync::Arc;

use zeloxy::Proxy;
use zeloxy::tools::ping_proxy_parallel;

#[tokio::main]
async fn main() {
  // Create a list of proxies
  let proxies = vec![
    "socks5://195.19.50.126:1080",
    "socks5://212.58.132.5:1080",
    "socks5://34.174.40.246:1080",
    "socks4://98.181.137.83:4145",
    "socks4://184.178.172.14:4145",
    "socks4://98.170.57.249:4145",
  ];

  let mut handles = Vec::new();

  for proxy_str in proxies {
    // Spawn a separate task for faster checking
    let handle = tokio::spawn(async move {
      // Create a proxy
      let proxy = Proxy::from(proxy_str);

      // Check the proxy using the default list of target services
      let result = ping_proxy_parallel(Arc::new(proxy), None).await;

      if result.pinged_services.len() > 0 {
      println!("\n============== {} ==============", proxy_str);

      // Log the results
      for (name, ping) in result.pinged_services {
        println!("|- Ping {}: {}ms", name, ping);
      }

      println!("|------------------------------------------------------");

      if let Some(average_ping) = result.average_ping {
      println!("|- Average proxy ping: {}ms", average_ping);
      }

      println!("==================================================================");
      } else {
      println!("\n[!] Proxy {} unavailable", proxy_str);
      }
    });

    handles.push(handle);
  }

  // Wait for all tasks to complete
  for handle in handles {
    let _ = handle.await;
  }
}
```

The program output will be something like this:

```
============== socks4://98.181.137.83:4145 ===============
|- Ping cloudflare.com: 558ms
|- Ping facebook.com: 638ms
|- Ping yandex.ru: 693ms
|- Ping github.com: 560ms
|- Ping reddit.com: 557ms
|-------------------------------------------------------
|- Average proxy ping: 601ms
============================================================

=============== socks5://212.58.132.5:1080 ==============
|- Ping reddit.com: 965ms
|- Ping cloudflare.com: 1007ms
|- Ping yandex.ru: 1162ms
|- Ping github.com: 1224ms
|- Ping facebook.com: 1251ms
|- Ping youtube.com: 1017ms
|-------------------------------------------------------
|- Average proxy ping: 1104ms
================================================================

And so on...
```

## Proxy lookup

Here we'll write a program that will obtain basic IP information.

```rust
use zeloxy::Proxy;
use zeloxy::tools::lookup_proxy;

#[tokio::main]
async fn main() {
  // Create a SOCKS4 proxy
  let proxy = Proxy::from("socks4://98.181.137.83:4145");

  // Logging proxy IP information
  if let Some(info) = lookup_proxy(&proxy).await {
    println!("IP information: {:#?}", info);
  } else {
    println!("Failed to get IP information");
  }
}
```

The output should look like this:

```
IP information: LookupInfo {
  ip: "98.181.137.83",
  hostname: "null",
  city: "Las Vegas",
  region: "Nevada",
  country: "US",
  location: "36.1750,-115.1372",
  organization: "AS22773 Cox Communications Inc.",
  postal: "89111",
  timezone: "America/Los_Angeles",
}
```