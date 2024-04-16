use std::{
    env, fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread, time::Duration
};

use http_server::pool::ThreadPool;

use crate::config::Config;
pub mod config;

fn main() {
    let conf = Config::parse(env::args());
    let address = format!("127.0.0.1:{}", conf.port());
    let listener = TcpListener::bind(address)
                               .expect("Could not bind to specified port");
    println!("Sever listening on port {}", conf.port());
    let pool = ThreadPool::new(4).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // let http_request: Vec<String> =
    //     buf_reader
    //     .lines()
    //     .map(Result::unwrap)
    //     .take_while(|l| !l.is_empty())
    //     .collect();
    // println!("Request: {:#?}", http_request);


    let request_line = buf_reader
        .lines()
        .next()
        .unwrap()
        .unwrap();


    let (status,filename) =
        match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK","hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            },
            _ => ("HTTP/1.1 404 NOT FOUND","404.html")
        };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!(
        "{status}\r\n\
        Content-Length: {length}\r\n\r\n\
        {contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();

}
