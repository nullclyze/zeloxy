//! **Zeloxy** - это небольшая библиотека, содержащая в себе логику создания прокси соединений, работающих
//! на разных популярных протоколах (например, HTTP, SOCKS5).
//!
//! ---
//!
//! ## Особенности
//!
//! - Поддержка базовой авторизации
//! - Асинхронная среда
//! - Прокси цепочки (смотри [раздел документации](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md#%D1%86%D0%B5%D0%BF%D0%BE%D1%87%D0%BA%D0%B0-%D0%BF%D1%80%D0%BE%D0%BA%D1%81%D0%B8))
//! - Встроенная реализация прокси потока (смотри [раздел документации](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md#%D0%BF%D0%B5%D1%80%D0%B2%D0%B0%D1%8F-%D0%BF%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B0))
//! - Пингования прокси (смотри [раздел документации](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md#%D0%BF%D0%B8%D0%BD%D0%B3%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D0%B5-%D0%BF%D1%80%D0%BE%D0%BA%D1%81%D0%B8))
//! - Поиск информации о прокси (смотри [раздел документации](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md#%D0%BF%D0%BE%D0%BB%D1%83%D1%87%D0%B5%D0%BD%D0%B8%D1%8F-%D0%B4%D0%B0%D0%BD%D0%BD%D1%8B%D1%85-%D0%BE%D0%B1-ip))
//!
//! ---
//!
//! ## Документация
//!
//! [Русская](https://github.com/nullclyze/zeloxy/blob/main/docs/RU.md) | [English](https://github.com/nullclyze/zeloxy/blob/main/docs/EN.md)
//!
//! ---
//!
//! ## Примеры
//!
//! Все актуальные примеры кода можно посмотреть здесь: [смотреть](https://github.com/nullclyze/zeloxy/tree/main/examples)

mod auth;
mod error;
mod result;
mod validate;

#[cfg(any(feature = "http", feature = "socks4", feature = "socks5"))]
mod connect;

#[cfg(any(feature = "http", feature = "socks4", feature = "socks5"))]
mod proxy;

#[cfg(feature = "chain")]
pub mod chain;

#[cfg(feature = "stream")]
mod stream;

#[cfg(feature = "tools")]
pub mod tools;

pub use auth::*;
pub use error::*;
pub use result::*;
pub use validate::*;

#[cfg(any(feature = "http", feature = "socks4", feature = "socks5"))]
pub use proxy::*;

#[cfg(any(feature = "http", feature = "socks4", feature = "socks5"))]
pub use connect::*;

#[cfg(feature = "chain")]
pub use chain::*;

#[cfg(feature = "stream")]
pub use stream::*;
