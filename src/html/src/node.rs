use std::{borrow::Cow, collections::HashMap};

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
pub struct HtmlNode<'a> {
    name: Cow<'a,str>,
    params: HashMap<Cow<'a,str>,Cow<'a,str>>,
    text: Cow<'a,str>,
    childs: Vec<HtmlNode<'a>>,
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
    /// Append an [HtmlNode] as a child
    #[inline]
    pub fn append(&mut self, node: HtmlNode<'a>) -> &mut Self {
        self.childs.push(node);
        self
    }
    /// Append all [nodes](HtmlNode) from the iterator as childs
    #[inline]
    pub fn append_iter(&mut self, node: impl Iterator<Item=HtmlNode<'a>>) -> &mut Self {
        node.for_each(|n| { self.append(n); });
        self
    }
    #[inline]
    pub fn append_to(self, other: &mut Self) -> &mut Self {
        other.append(self);
        other
    }
    /// Create an [HtmlNode] with a given name
    #[inline]
    pub fn with_name(name: impl Into<Cow<'a,str>>) -> Self {
        let mut node = HtmlNode::new();
        node.name = name.into();
        node
    }
    /// Set the text inside the node
    #[inline]
    pub fn text(&mut self, text: impl Into<Cow<'a,str>>) -> &mut Self {
        self.text = text.into();
        self
    }
    /// Set an attribute of the [HtmlNode]
    #[inline]
    pub fn attr(&mut self, name: impl Into<Cow<'a,str>>, value: impl Into<Cow<'a,str>>) -> &mut Self {
        self.params.insert(name.into(),value.into());
        self
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
