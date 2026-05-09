use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

use crate::stream::reader::ProxyReader;
use crate::stream::writer::ProxyWriter;
use crate::{ErrorKind, Proxy, ProxyError, ProxyResult};

/// TCP-соединение с прокси
pub struct ProxyStream {
  proxy: Proxy,
  reader: Mutex<Option<ProxyReader>>,
  writer: Mutex<Option<ProxyWriter>>,
}

impl ProxyStream {
  /// Метод создания `ProxyStream` (данный метод не подключается к целевому
  /// серверу через прокси, для этого используется метод `ProxyStream::connect`)
  pub fn new(proxy: Proxy) -> Self {
    Self {
      proxy,
      reader: Mutex::new(None),
      writer: Mutex::new(None),
    }
  }

  /// Метод подключения к целевому серверу через прокси
  pub async fn connect(&self, target_host: impl Into<String>, target_port: u16) -> ProxyResult<()> {
    let stream = self.proxy.connect(target_host, target_port).await?;
    let (rh, wh) = stream.into_split();

    *self.reader.lock().await = Some(ProxyReader { read_stream: rh });
    *self.writer.lock().await = Some(ProxyWriter { write_stream: wh });

    Ok(())
  }

  /// Метод выключения текущего TCP-соединения
  pub async fn shutdown(&self) -> ProxyResult<()> {
    if let Some(writer) = self.writer.lock().await.as_mut() {
      writer.shutdown().await?;
    }

    *self.reader.lock().await = None;
    *self.writer.lock().await = None;

    Ok(())
  }

  /// Метод чтения данных из потока
  pub async fn read(&self, buffer: impl Into<&mut [u8]>) -> ProxyResult<usize> {
    if let Some(reader) = self.reader.lock().await.as_mut() {
      Ok(reader.read(buffer).await?)
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "reader is not initialized"))
    }
  }

  /// Метод записи данных в поток
  pub async fn write(&self, buffer: impl Into<&[u8]>) -> ProxyResult<()> {
    if let Some(writer) = self.writer.lock().await.as_mut() {
      Ok(writer.write(buffer).await?)
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "writer is not initialized"))
    }
  }

  /// Вспомогательный метод отправки команды GET-запроса
  pub async fn get_request(&self, host: impl Into<String>) -> ProxyResult<String> {
    if let Some(writer) = self.writer.lock().await.as_mut() {
      let req = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host.into());
      writer.write(req.as_bytes()).await?;
    } else {
      return Err(ProxyError::new(ErrorKind::StreamError, "writer is not initialized"));
    }

    if let Some(reader) = self.reader.lock().await.as_mut() {
      let mut resp = Vec::new();
      reader.read_stream.read_to_end(&mut resp).await?;

      Ok(String::from_utf8_lossy(&resp).to_string())
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "reader is not initialized"))
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{Proxy, ProxyResult, ProxyStream};

  #[tokio::test]
  async fn test_proxy_stream() -> ProxyResult<()> {
    let proxy = Proxy::from("socks4://68.71.242.118:4145");
    let stream = ProxyStream::new(proxy);

    stream.connect("ipinfo.io", 80).await?;

    let resp = stream.get_request("ipinfo.io").await?;

    println!("Ответ: {}", resp);

    Ok(())
  }
}
