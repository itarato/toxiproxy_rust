//! Toxiproxy-Rust is a [Toxiproxy] client written for the Rust language.
//! It's designed for testing network code and its resiliency against various
//! network issues, such as latency or unavailability (and many more).
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
