#![feature(test)]

use rhtml::{HtmlBuilder, HtmlNode};
use test::Bencher;
extern crate test;

const N: u32 = 20000;

#[bench]
fn owned(b: &mut Bencher) {
    b.iter(|| {
        let mut builder = HtmlBuilder::new();
        for _ in 0..N {
            let mut node = HtmlNode::with_name("p");
            node.text("Hello world!".to_owned());
            node.attr("ATTTR".to_owned(), "VALLL".to_owned());
            builder.body().append(node);
        }
        builder.to_string();
    });
}

#[bench]
fn reference(b: &mut Bencher) {
    b.iter(|| {
        let mut builder = HtmlBuilder::new();
        for _ in 0..N {
            let mut node = HtmlNode::with_name("p");
            node.text("Hello world!");
            node.attr("ATTTR", "VALLL");
            builder.body().append(node);
        }
        builder.to_string();
    });
}
