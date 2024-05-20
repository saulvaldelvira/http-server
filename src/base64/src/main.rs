use rb64::*;

fn main() {
    let enc = encode(b"Hello world!");
    println!("Encoded: {enc}");
    let dec = decode(&enc).unwrap();
    let dec = String::from_utf8(dec).unwrap();
    println!("Decoded: {dec}");
}
