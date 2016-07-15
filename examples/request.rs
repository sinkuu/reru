extern crate reru;
extern crate serde_json;

fn main() {
    let json: serde_json::Value = reru::post("https://httpbin.org/post")
        .expect("failed to parse URL")
        .param("show_env", "1")
        .body_json(&["蟹", "Ferris"])
        .expect("failed to serialize")
        .request()
        .expect("failed to send request")
        .parse_json()
        .expect("failed to parse JSON");

    println!("{}", json);
}
