#![feature(test)]

use rhtml::{HtmlBuilder, HtmlNode};
use test::Bencher;
extern crate test;

#[bench]
fn owned(b: &mut Bencher) {
    let mut builder = HtmlBuilder::new();
    b.iter(|| {
        let mut node = HtmlNode::with_name("p");
        node.text("Hello world!".to_string());
        node.attr("ATTTR".to_string(), "VALLL".to_string());
        builder.body().append(node);
    });
}

#[bench]
fn reference(b: &mut Bencher) {
    let mut builder = HtmlBuilder::new();
    b.iter(|| {
        let mut node = HtmlNode::with_name("p");
        node.text("Hello world!");
        node.attr("ATTTR", "VALLL");
        builder.body().append(node);
    });
}
