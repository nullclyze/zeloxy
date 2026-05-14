use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

/// Отдельная рука для чтения данных из потока
pub struct ProxyReader {
  pub read_stream: OwnedReadHalf,
}

impl ProxyReader {
  /// Метод чтения буффера из потока
  pub async fn read(&mut self, buffer: impl Into<&mut [u8]>) -> std::io::Result<usize> {
    self.read_stream.read_exact(buffer.into()).await
  }

  /// Метод чтения буффера из потока до конца
  pub async fn read_to_end(&mut self, buffer: impl Into<&mut Vec<u8>>) -> std::io::Result<usize> {
    self.read_stream.read_to_end(buffer.into()).await
  }
}
