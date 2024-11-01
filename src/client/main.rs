use std::{env, net::{TcpStream, ToSocketAddrs}};
use super::ClientConfig;

use crate::request::{HttpRequest, RequestMethod};


pub fn main() {
    let conf = ClientConfig::parse(env::args().skip(1)).unwrap();

    let addrs = conf.host.to_socket_addrs().unwrap().next().unwrap();

    let req = HttpRequest::builder()
                .method(RequestMethod::GET)
                .url(conf.url)
                .version(1.1)
                .header("Host", conf.host)
                .header("Accept", "*/*")
                .header("User-Agent", "http-client")
                .build().unwrap();
    let tcp = TcpStream::connect(addrs).unwrap();
    let mut result = req.send_to(tcp).unwrap();

    let s = String::from_utf8(result.body().to_vec()).unwrap();
    println!("{s}");
}
