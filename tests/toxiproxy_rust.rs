#![deny(warnings)]

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
    let result = TOXIPROXY.populate(vec![Proxy::new(
        "socket".into(),
        "localhost:2001".into(),
        "localhost:2000".into(),
    )]);

    assert!(result.is_ok());

    assert_eq!(1, result.as_ref().unwrap().len());
    assert_eq!("socket", result.as_ref().unwrap()[0].name);
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

    assert_eq!("socket", result.as_ref().unwrap().name);
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
    assert!(result.as_ref().unwrap().enabled);

    assert!(result
        .as_ref()
        .unwrap()
        .down(|| {
            let result = TOXIPROXY.find_proxy("socket");
            assert!(result.is_ok());
            assert!(!result.as_ref().unwrap().enabled);
            let _ = !result.as_ref().unwrap().enabled;
        })
        .is_ok());

    let result = TOXIPROXY.find_proxy("socket");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().enabled);
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

/**
 * Support functions.
 */

fn populate_example() {
    let result = TOXIPROXY.populate(vec![Proxy::new(
        "socket".into(),
        "localhost:2001".into(),
        "localhost:2000".into(),
    )]);

    assert!(result.is_ok());
}

/*

// use std::io::prelude::*;
// use std::net::TcpStream;
// use std::time::SystemTime;

// println!("START {:?}", SystemTime::now());

// // dbg!(TOXIPROXY.all());

// let mut stream =
//     TcpStream::connect("localhost:2001").expect("stream cannot be created");

// let mut out = String::new();

// stream.read_to_string(&mut out).expect("read body failed");

// // dbg!(out);
// println!("END {:?}", SystemTime::now());
 */
