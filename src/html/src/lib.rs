pub mod node;
pub use node::HtmlNode;
pub mod builder;
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

