use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::stream::reader::ProxyReader;
use crate::stream::writer::ProxyWriter;
use crate::{ErrorKind, Proxy, ProxyError, ProxyResult};

/// TCP-соединение с прокси
pub struct ProxyStream {
  pub reader: Mutex<Option<ProxyReader>>,
  pub writer: Mutex<Option<ProxyWriter>>,
  proxy: Option<Proxy>,
}

impl ProxyStream {
  /// Метод создания `ProxyStream` (данный метод не подключается к целевому
  /// серверу через прокси, для этого используется метод `ProxyStream::connect`)
  pub fn new(proxy: impl Into<Proxy>) -> Self {
    Self {
      proxy: Some(proxy.into()),
      reader: Mutex::new(None),
      writer: Mutex::new(None),
    }
  }

  /// Метод создания `ProxyStream` с ранее созданным `TcpStream`
  pub fn new_with_stream(stream: impl Into<TcpStream>) -> Self {
    let (rh, wh) = stream.into().into_split();

    Self {
      proxy: None,
      reader: Mutex::new(Some(ProxyReader { read_stream: rh })),
      writer: Mutex::new(Some(ProxyWriter { write_stream: wh })),
    }
  }

  /// Метод установки прокси
  pub fn set_proxy(&mut self, proxy: impl Into<Proxy>) {
    self.proxy = Some(proxy.into());
  }

  /// Метод подключения к целевому серверу через прокси
  pub async fn connect(&self, target_host: impl Into<String>, target_port: u16) -> ProxyResult<()> {
    if let Some(proxy) = &self.proxy {
      let stream = proxy.connect(target_host, target_port).await?;
      let (rh, wh) = stream.into_split();

      *self.reader.lock().await = Some(ProxyReader { read_stream: rh });
      *self.writer.lock().await = Some(ProxyWriter { write_stream: wh });
    } else {
      return Err(ProxyError::new(ErrorKind::InvalidData, "proxy not set"));
    }

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
  pub async fn read(&self, buffer: &mut [u8]) -> ProxyResult<usize> {
    if let Some(reader) = self.reader.lock().await.as_mut() {
      Ok(reader.read(buffer).await?)
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "reader is not initialized"))
    }
  }

  /// Метод чтения данных из потока до конца
  pub async fn read_to_end(&self, buffer: &mut Vec<u8>) -> ProxyResult<usize> {
    if let Some(reader) = self.reader.lock().await.as_mut() {
      Ok(reader.read_to_end(buffer).await?)
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "reader is not initialized"))
    }
  }

  /// Метод записи данных в поток
  pub async fn write(&self, buffer: &[u8]) -> ProxyResult<()> {
    if let Some(writer) = self.writer.lock().await.as_mut() {
      Ok(writer.write(buffer).await?)
    } else {
      Err(ProxyError::new(ErrorKind::StreamError, "writer is not initialized"))
    }
  }
}

impl From<TcpStream> for ProxyStream {
  fn from(value: TcpStream) -> Self {
    Self::new_with_stream(value)
  }
}
