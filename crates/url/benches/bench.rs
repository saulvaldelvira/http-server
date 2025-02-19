#![feature(test)]
extern crate test;

use test::Bencher;

const N: u32 = 5000;

#[bench]
fn encode_nop(b: &mut Bencher) {
    let mut s = String::new();
    for _ in 0..N {
        for c in 'A'..'z' {
            s.push(c);
        }
    }
    b.iter(|| {
        url::encode(&s).unwrap();
    });
}

#[bench]
fn encode(b: &mut Bencher) {
    let mut s = String::new();
    for _ in 0..N {
        for c in 'A'..'x' {
            s.push(c);
        }
        s.push('ñ');
    }
    b.iter(|| {
        url::encode(&s).unwrap();
    });
}

#[bench]
fn decode_nop(b: &mut Bencher) {
    let mut s = String::new();
    for _ in 0..N {
        for c in 'A'..'z' {
            s.push(c);
        }
    }
    b.iter(|| {
        url::decode(&s).unwrap();
    });
}

#[bench]
fn decode(b: &mut Bencher) {
    let mut s = String::new();
    for _ in 0..N {
        for c in 'A'..'x' {
            s.push(c);
        }
        s.push('+');
    }
    b.iter(|| {
        url::decode(&s).unwrap();
    });
}
