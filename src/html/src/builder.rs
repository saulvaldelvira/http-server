use std::{borrow::Cow, fmt::Display};

use crate::{HtmlNode,html};

/// Html Builder
///
/// This struct represents an Html Document.
pub struct HtmlBuilder<'a> {
    root: HtmlNode<'a>,
}

impl<'a> HtmlBuilder<'a> {
    /// Create a new HTML Builder
    ///
    /// By default the following document is created.
    /// ```html
    /// <html>
    ///     <head>
    ///         <meta charset="UTF-8"></meta>
    ///     </head>
    ///     <body></body>
    /// </html>
    /// ```
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
    #[inline]
    pub fn with_title(title: impl Into<Cow<'a,str>>) -> Self {
        let mut builder = Self::new();
        builder.head().append(html!("title", {text: title}));
        builder
    }
    #[inline]
    pub fn head(&mut self) -> &mut HtmlNode<'a> {
        self.root.nth(0).unwrap()
    }
    #[inline]
    pub fn body(&mut self) -> &mut HtmlNode<'a> {
        self.root.nth(1).unwrap()
    }
}

impl<'a> Default for HtmlBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for HtmlBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!DOCTYPE html>")?;
        self.root.write_to(f)
    }
}
