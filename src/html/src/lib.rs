//! This crate contains code to build Html Pages
//!
//! # Example
//! ```
//! use rhtml::*;
//!
//! let mut builder = HtmlBuilder::with_title("My page");
//! let body = builder.body();
//! html!("h1", {text: "Hello world!"}).append_to(body);
//! html!("a", {"href": "http://www.web.net"},
//!            {text: "My link"}).append_to(body);
//! let page = builder.to_string();
//! ```

mod node;
pub use node::HtmlNode;
mod builder;
pub use builder::HtmlBuilder;

#[macro_export]
macro_rules! html {
    ($name:literal /* Name of the node */
     /* Attributes */
     $(, {
         $(
             $attr:literal : $value:expr
          ),*
     }
     )?
     /* Functions */
     $(, {
         $(
             $f:ident : $v:expr
          ),*
     }
     )?
     /* Childs */
     $(,
       [ $($o:expr),* $(,)? ]
      )?
    ) => {
        {
            #[allow(unused_mut)]
            let mut node = HtmlNode::with_name($name);
            $(
                $(
                    node.attr($attr,$value);
                 )*
             )?
                $(
                    $(
                        node.$f($v);
                     )*
                 )?
                $(
                    $(
                        node.append($o);
                     )*
                 )?
                node
        }
    };
}

