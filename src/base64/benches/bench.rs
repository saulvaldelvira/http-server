#![feature(test)]
extern crate test;
use std::u8;

use test::Bencher;

#[bench]
fn bench_encode(b: &mut Bencher) {
    let mut s = Vec::new();
    for _ in 0..10000 {
        for i in 0..u8::MAX {
            s.push(i);
        }
    }
    b.iter(|| {
        rb64::encode(&s);
    });
}

#[bench]
fn bench_decode(b: &mut Bencher) {
    let mut s = Vec::new();
    for _ in 0..10000 {
        for i in 0..u8::MAX {
            s.push(i);
        }
    }
    let enc = rb64::encode(&s);
    b.iter(|| {
        rb64::decode(&enc).unwrap();
    })
}
