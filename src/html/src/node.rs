use std::collections::HashMap;

/// Html Node
///
/// Node of an HTML Document
///
/// # Example
/// ```rust
/// use rhtml::*;
///
/// let node = html!("ul", [
///     html!("li", {text: "Element1"}),
///     html!("li", {text: "Element2"}),
///     html!("li", {text: "Element3"}),
/// ]);
/// ```
pub struct HtmlNode {
    name: String,
    params: HashMap<String,String>,
    text: String,
    childs: Vec<HtmlNode>,
}

impl HtmlNode {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            params: HashMap::new(),
            childs: Vec::new(),
            name: String::new(),
        }
    }
    /// Append an [HtmlNode] as a child
    #[inline]
    pub fn append(&mut self, node: HtmlNode) -> &mut Self {
        self.childs.push(node);
        self
    }
    /// Append all [nodes](HtmlNode) from the iterator as childs
    #[inline]
    pub fn append_iter(&mut self, node: impl Iterator<Item=HtmlNode>) -> &mut Self {
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
    pub fn with_name(name: &str) -> Self {
        let mut node = HtmlNode::new();
        node.name = name.to_owned();
        node
    }
    /// Set the text inside the node
    #[inline]
    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        self.text = text.into();
        self
    }
    /// Set an attribute of the [HtmlNode]
    #[inline]
    pub fn attr(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.params.insert(name.into(),value.into());
        self
    }
    /// Get the n'th child
    #[inline]
    pub fn nth(&mut self, i: usize) -> Option<&mut Self> {
        self.childs.get_mut(i)
    }
}

impl ToString for HtmlNode {
    fn to_string(&self) -> String {
        let mut text = "".to_string();
        text = text + "<" + &self.name;
        for (k,v) in &self.params {
            text = text + " " + k + "=\"" + v + "\"";
        }
        text += ">";
        if !self.text.is_empty() {
            text = text + &self.text;
        }
        for child in &self.childs {
            text += &child.to_string();
        }
        text = text + "</" + &self.name + ">";
        text
    }
}
