# reru

[![Crates.io](https://img.shields.io/crates/v/reru.svg?maxAge=2592000)](https://crates.io/crates/reru)

A simple HTTP request client for Rust, inspired by [unirest](http://unirest.io/).

[Documentation](https://sinkuu.github.io/reru/reru/)

```rust
let mut res = String::new();

reru::post("https://httpbin.org/post")
    .expect("failed to parse URL")
    .param("show_env", "1")
    .body_json(&["蟹", "Ferris"])
    .expect("failed to serialize")
    .request()
    .expect("failed to send request")
    .read_to_string(&mut res)
    .expect("failed to read response");

println!("{}", res);
```
