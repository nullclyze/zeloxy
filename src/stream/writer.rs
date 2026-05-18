use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

/// Отдельная рука для записи данных в поток
pub struct ProxyWriter {
  pub write_stream: OwnedWriteHalf,
}

impl ProxyWriter {
  /// Метод выключения потока записи
  pub async fn shutdown(&mut self) -> std::io::Result<()> {
    self.write_stream.shutdown().await
  }

  /// Метод записи буффера в поток
  pub async fn write(&mut self, buffer: &[u8]) -> std::io::Result<()> {
    self.write_stream.write_all(buffer).await
  }
}
