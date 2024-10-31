use std::net::{TcpStream, ToSocketAddrs};
use http_srv::request::{HttpRequest, RequestMethod};

fn main() {
    let req = HttpRequest::builder()
                .method(RequestMethod::GET)
                .url("/")
                .version(1.1)
                .header("Host", "")
                .header("Accept", "*/*")
                .header("User-Agent", "Pepe")
                .build().unwrap();
    let addrs = "git.saulv.es:80".to_socket_addrs().unwrap().next().unwrap();
    let tcp = TcpStream::connect(addrs).unwrap();
    let mut result = req.send_to(tcp).unwrap();

    let s = String::from_utf8(result.body().to_vec()).unwrap();
    println!("{s}");
}
