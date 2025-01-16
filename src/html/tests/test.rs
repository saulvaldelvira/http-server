use rhtml::*;

#[test]
fn test() {
    let mut builder = HtmlBuilder::new();
    builder.body().append(html!("h1", {text: "Hello World!"}));
    assert_eq!(
        "<!DOCTYPE html>\
        <html><head><meta charset=\"UTF-8\"></meta></head>\
        <body><h1>Hello World!</h1></body></html>",
        builder.to_string()
    );
}
