#![deny(warnings)]

use std::net::TcpListener;
use std::net::TcpStream;
use std::thread::spawn;
use std::time::SystemTime;
use std::{io::prelude::*, time::Duration};

use proxy::*;
use toxiproxy_rust::*;

/**
 * WARNING!!!: This test depends on Toxiproxy (https://github.com/Shopify/toxiproxy) server running locally on default port.
 */

#[test]
fn test_is_running() {
    assert!(TOXIPROXY.is_running());
}

#[test]
fn test_reset() {
    assert!(TOXIPROXY.reset().is_ok());
}

#[test]
fn test_populate() {
    let result = TOXIPROXY.populate(vec![ProxyPack::new(
        "socket".into(),
        "localhost:2001".into(),
        "localhost:2000".into(),
    )]);

    assert!(result.is_ok());

    assert_eq!(1, result.as_ref().unwrap().len());
    assert_eq!("socket", result.as_ref().unwrap()[0].proxy_pack.name);
}

#[test]
fn test_all() {
    populate_example();

    let result = TOXIPROXY.all();
    assert!(result.is_ok());
    assert_eq!(1, result.as_ref().unwrap().len());
}

#[test]
fn test_version() {
    assert!(TOXIPROXY.version().is_ok());
}

#[test]
fn test_find_proxy() {
    populate_example();

    let result = TOXIPROXY.find_proxy("socket");
    assert!(result.is_ok());

    assert_eq!("socket", result.as_ref().unwrap().proxy_pack.name);
}

#[test]
fn test_find_proxy_invalid() {
    let result = TOXIPROXY.find_proxy("bad-proxy");
    assert!(result.is_err());
}

#[test]
fn test_proxy_down() {
    populate_example();

    let result = TOXIPROXY.find_proxy("socket");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().proxy_pack.enabled);

    assert!(result
        .as_ref()
        .unwrap()
        .with_down(|| {
            let result = TOXIPROXY.find_proxy("socket");
            assert!(result.is_ok());
            assert!(!result.as_ref().unwrap().proxy_pack.enabled);
        })
        .is_ok());

    let result = TOXIPROXY.find_proxy("socket");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().proxy_pack.enabled);
}

#[test]
fn test_proxy_apply_with_latency() {
    populate_example();

    let proxy_result = TOXIPROXY.find_proxy("socket");
    assert!(proxy_result.is_ok());

    let proxy_toxics = proxy_result.as_ref().unwrap().toxics();
    assert!(proxy_toxics.is_ok());
    assert_eq!(0, proxy_toxics.as_ref().unwrap().len());

    let apply_result = proxy_result
        .as_ref()
        .unwrap()
        .with_latency("downstream".into(), 2000, 0, 1.0)
        .apply(|| {
            let all = TOXIPROXY.all();
            assert!(all.is_ok());
            let proxy = all.as_ref().unwrap().get("socket");
            assert!(proxy.is_some());

            let proxy_toxics = proxy.as_ref().unwrap().toxics();
            assert!(proxy_toxics.is_ok());
            assert_eq!(1, proxy_toxics.as_ref().unwrap().len());
        });

    assert!(apply_result.is_ok());

    let proxy_toxics = proxy_result.as_ref().unwrap().toxics();
    assert!(proxy_toxics.is_ok());
    assert_eq!(0, proxy_toxics.as_ref().unwrap().len());
}

#[test]
fn test_proxy_apply_with_latency_as_separate_calls_for_test() {
    populate_example();

    let proxy_result = TOXIPROXY.find_proxy("socket");
    assert!(proxy_result.is_ok());

    let proxy_toxics = proxy_result.as_ref().unwrap().toxics();
    assert!(proxy_toxics.is_ok());
    assert_eq!(0, proxy_toxics.as_ref().unwrap().len());

    let _ = proxy_result
        .as_ref()
        .unwrap()
        .with_latency("downstream".into(), 2000, 0, 1.0);

    let all = TOXIPROXY.all();
    assert!(all.is_ok());
    let proxy = all.as_ref().unwrap().get("socket");
    assert!(proxy.is_some());

    let proxy_toxics = proxy.as_ref().unwrap().toxics();
    assert!(proxy_toxics.is_ok());
    assert_eq!(1, proxy_toxics.as_ref().unwrap().len());
}

#[test]
fn test_proxy_apply_with_latency_with_real_request() {
    let server_thread = spawn(|| one_take_server());
    populate_example();

    let proxy_result = TOXIPROXY.find_proxy("socket");
    assert!(proxy_result.is_ok());

    let apply_result = proxy_result
        .as_ref()
        .unwrap()
        .with_latency("downstream".into(), 2000, 0, 1.0)
        .apply(|| {
            let client_thread = spawn(|| one_shot_client());

            server_thread.join().expect("Failed closing server thread");
            let duration = client_thread.join().expect("Failed closing client thread");

            assert!(duration.as_secs() >= 2);
        });

    assert!(apply_result.is_ok());
}

/**
 * Support functions.
 */

fn populate_example() {
    let result = TOXIPROXY.populate(vec![ProxyPack::new(
        "socket".into(),
        "localhost:2001".into(),
        "localhost:2000".into(),
    )]);

    assert!(result.is_ok());
}

fn one_shot_client() -> Duration {
    let t_start = SystemTime::now();

    let mut stream = TcpStream::connect("localhost:2001").expect("Failed to connect to server");

    stream
        .write("hello".as_bytes())
        .expect("Client failed sending request");

    stream
        .read(&mut [0u8; 1024])
        .expect("Client failed reading response");

    t_start.elapsed().expect("Cannot establish duration")
}

fn one_take_server() {
    let mut stream = TcpListener::bind("localhost:2000")
        .expect("TcpListener cannot connect")
        .incoming()
        .next()
        .expect("Failed to listen for incoming")
        .expect("Request failes");

    stream
        .read(&mut [0u8; 1024])
        .expect("Server failed reading request");

    stream
        .write("byebye".as_bytes())
        .expect("Server failed writing response");

    stream.flush().expect("Failed flushing connection");
}
