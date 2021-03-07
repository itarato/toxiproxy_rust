#[macro_use]
extern crate lazy_static;

pub mod client;
pub mod consts;
pub mod http_client;
pub mod proxy;
mod toxic;

use client::*;
use consts::*;

lazy_static! {
    pub static ref TOXIPROXY: Client = Client::new(TOXIPROXY_DEFAULT_URI.into());
}
