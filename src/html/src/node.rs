use std::collections::HashMap;

/// Html Node
///
/// Node of an HTML Document
///
/// # Example
/// ```rust
/// use crate::html::*;
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
    pub fn append(&mut self, node: HtmlNode) -> &mut Self {
        self.childs.push(node);
        self
    }
    /// Append all [nodes](HtmlNode) from the iterator as childs
    pub fn append_iter(&mut self, node: impl Iterator<Item=HtmlNode>) -> &mut Self {
        node.for_each(|n| { self.append(n); });
        self
    }
    /// Create an [HtmlNode] with a given name
    pub fn with_name(name: &str) -> Self {
        let mut node = HtmlNode::new();
        node.name = name.to_owned();
        node
    }
    /// Set the text inside the node
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.text.clear();
        self.text.push_str(text);
        self
    }
    /// Set an attribute of the [HtmlNode]
    pub fn attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.params.insert(name.to_owned(),value.to_owned());
        self
    }
    /// Get the n'th child
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

