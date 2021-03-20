//! Toxiproxy-Rust is a [Toxiproxy] client written for the Rust language.
//! It's designed for testing network code and its resiliency against various
//! network issues, such as latency or unavailability (and many more).
//!
//! ## Setting up a test
//!
//! ```rust
//! use toxiproxy_rust::{TOXIPROXY, proxy::ProxyPack};
//!
//! TOXIPROXY.populate(vec![ProxyPack::new(
//!     "socket".into(),
//!     "localhost:2001".into(),
//!     "localhost:2000".into(),
//! )]);
//!
//! TOXIPROXY
//!     .find_and_reset_proxy("socket")
//!     .unwrap()
//!     .with_down(|| {
//!         /* For example:
//!         let result = MyService::Server.call();
//!         assert!(result.is_ok());
//!         */
//!     });
//! ```
//!
//! ## Setting up a more advanced test
//!
//! ```rust
//! use toxiproxy_rust::{TOXIPROXY, proxy::ProxyPack};
//!
//! TOXIPROXY.populate(vec![ProxyPack::new(
//!     "socket".into(),
//!     "localhost:2001".into(),
//!     "localhost:2000".into(),
//! )]);
//!
//! TOXIPROXY
//!     .find_and_reset_proxy("socket")
//!     .unwrap()
//!     .with_slicer("downstream".into(), 2048, 128, 0, 0.8)
//!     .with_bandwidth("downstream".into(), 32, 0.5)
//!     .apply(|| {
//!         /* For example:
//!         let result = MyService::Server.call();
//!         assert!(result.is_ok());
//!         */
//!     });
//! ```
//!
//! [Toxiproxy]: https://github.com/Shopify/toxiproxy

#[macro_use]
extern crate lazy_static;

pub mod client;
mod consts;
mod http_client;
pub mod proxy;
pub mod toxic;

use client::*;

lazy_static! {
    /// Pre-built client using the default connection address.
    pub static ref TOXIPROXY: Client = Client::new("127.0.0.1:8474");
}
