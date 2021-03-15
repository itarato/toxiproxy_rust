Toxiproxy - Rust client
-----------------------

[under development]

Rust client for [Toxiproxy](https://github.com/Shopify/toxiproxy).

## Usage

Populating proxies:

```rust
let proxies = TOXIPROXY.populate(vec![
  Proxy::new(
    "socket_service".into(),
    "localhost:2001".into(),
    "localhost:2000".into(),
  ),
  Proxy::new(
    "redis".into(),
    "localhost:6000".into(),
    "localhost:6379".into(),
  )
])?;
```

Testing with an unavailable connection:

```rust
TOXIPROXY.find_proxy("redis")?.down(|| {
  // Calling the desired service...
})?;
```

Testing with toxics (for full documentation on available toxics see [the original docs](https://github.com/Shopify/toxiproxy#toxics)):

```rust
TOXIPROXY.find_proxy("redis")?.with_latency("downstream".into(), 2000, 0, 1.0).apply(|| {
  // Calling the desired service...
})?;
```

Using a custom address for Toxiproxy server:

```rust
let toxiclient: Client = toxiproxy_rust::Client::new("1.2.3.4:5678");
```

## Development

Tests:

```bash
$> cargo test -- --test-threads 1
```
