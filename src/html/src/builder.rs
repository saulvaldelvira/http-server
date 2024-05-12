use crate::{HtmlNode,html};

/// Html Builder
///
/// This struct represents an Html Document.
pub struct HtmlBuilder {
    root: HtmlNode,
}

impl HtmlBuilder {
    /// Create a new HTML Builder
    ///
    /// By default the following document is created.
    /// <html>
    ///     <head>
    ///         <meta charset="UTF-8"></meta>
    ///     </head>
    ///     <body></body>
    /// </html>
    pub fn new() -> Self {
        let root =
            html!("html",
                  [
                    html!("head",
                          [
                            html!("meta", {"charset": "UTF-8"})
                          ]),
                    html!("body")
                  ]);
        Self { root }
    }
    /// Create a new [HtmlBuilder] with a title
    pub fn with_title(title: &str) -> Self {
        let mut builder = Self::new();
        builder.head().append(html!("title", {text: title}));
        builder
    }
    pub fn head(&mut self) -> &mut HtmlNode {
        self.root.nth(0).unwrap()
    }
    pub fn body(&mut self) -> &mut HtmlNode {
        self.root.nth(1).unwrap()
    }
}

impl ToString for HtmlBuilder {
    /// To generate the [String] representation of the document, the
    /// builder conctenates the "!DOCTYPE html" string and then calls
    /// the [to_string](HtmlNode::to_string) method of the root node.
    fn to_string(&self) -> String {
        "<!DOCTYPE html>".to_string() + &self.root.to_string()
    }
}
