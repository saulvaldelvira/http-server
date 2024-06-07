#![feature(test)]
extern crate test;
use test::Bencher;

use rmime::Mime;

const N: usize = 2048;

#[bench]
fn reference(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::from_str("text/plain").unwrap();
            v.push(mime);
        }
    });
}

#[bench]
fn from_owned(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::from_str("text/plain".to_owned()).unwrap();
            v.push(mime);
        }
    });
}
#[bench]
fn from_owned_and_into(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::from_str("text/plain".to_owned()).unwrap().into_owned();
            v.push(mime);
        }
    });
}

#[bench]
fn into_owned(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::from_str("text/plain").unwrap().into_owned();
            v.push(mime);
        }
    });
}
