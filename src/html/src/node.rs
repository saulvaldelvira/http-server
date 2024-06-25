use std::{borrow::Cow, collections::HashMap};

use builders::Builder;

/// Html Node
///
/// Node of an HTML Document
///
/// # Example
/// ```rust
/// use rhtml::*;
///
/// let mut builder = HtmlBuilder::new();
/// let mut node = html!("ul", [
///     html!("li", {text: "Element1"}),
///     html!("li", {text: "Element2"}),
///     html!("li", {text: "Element3"}),
/// ]);
/// node.attr("class", "my_list");
/// builder.body().append(node);
/// ```
#[derive(Builder,Clone)]
pub struct HtmlNode<'a> {
    name: Cow<'a,str>,
    #[builder(each = "attr")]
    params: HashMap<Cow<'a,str>,Cow<'a,str>>,
    text: Cow<'a,str>,
    #[builder(each = "child")]
    childs: Vec<HtmlNode<'a>>,
}

impl<'a> HtmlNodeBuilder<'a> {
    /// Append all [nodes](HtmlNode) from the iterator as childs
    #[inline]
    pub fn append_iter(&mut self, node: impl Iterator<Item=HtmlNode<'a>>) -> &mut Self {
        node.for_each(|n| { self.child(n); });
        self
    }
}

impl<'a> HtmlNode<'a> {
    pub fn new() -> Self {
        Self {
            text: "".into(),
            params: HashMap::new(),
            childs: Vec::new(),
            name: "".into()
        }
    }
    #[inline]
    pub fn append_to(self, other: &'a mut HtmlNodeBuilder<'a>) -> &mut HtmlNodeBuilder<'a> {
        other.child(self);
        other
    }
    /// Create an [HtmlNode] with a given name
    #[inline]
    pub fn with_name(name: impl Into<Cow<'a,str>>) -> HtmlNodeBuilder<'a> {
        let mut node = HtmlNode::builder();
        node.name(name.into());
        node
    }
    /// Get the n'th child
    #[inline]
    pub fn nth(&mut self, i: usize) -> Option<&mut HtmlNode<'a>> {
        self.childs.get_mut(i)
    }
    pub (crate) fn write_to(&self, buf: &mut String) {
        buf.push('<');
        buf.push_str(&self.name);
        for (k,v) in &self.params {
            buf.push(' ');
            buf.push_str(k);
            buf.push_str("=\"");
            buf.push_str(v);
            buf.push_str("\"");
        }
        buf.push('>');
        if !self.text.is_empty() {
            buf.push_str(&self.text);
        }
        for child in &self.childs {
            child.write_to(buf);
        }
        buf.push_str("</");
        buf.push_str(&self.name);
        buf.push('>');
    }
}

impl ToString for HtmlNode<'_> {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        self.write_to(&mut buf);
        buf
    }
 }
