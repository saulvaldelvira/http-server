#![feature(test)]
extern crate test;
use rmime::Mime;
use test::Bencher;

const N: usize = 2048;

#[bench]
fn reference(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::new("text/plain").unwrap();
            v.push(mime);
        }
    });
}

#[bench]
fn from_owned(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::new("text/plain".to_owned()).unwrap();
            v.push(mime);
        }
    });
}
#[bench]
fn from_owned_and_into(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::new("text/plain".to_owned()).unwrap().into_owned();
            v.push(mime);
        }
    });
}

#[bench]
fn into_owned(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            let mime = Mime::new("text/plain").unwrap().into_owned();
            v.push(mime);
        }
    });
}
