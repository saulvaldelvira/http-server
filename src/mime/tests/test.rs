use rmime::Mime;

#[test]
fn test() {
    const TESTS: [(&str, &str); 2] = [("video.mp4", "video/mp4"), ("audio.aac", "audio/aac")];
    for (name, expected) in TESTS {
        let mime = Mime::from_filename(name).unwrap();
        assert_eq!(mime.to_string(), expected);
    }
}

#[test]
fn no_ext() {
    let mime = Mime::from_filename("my_file").unwrap();
    assert_eq!("text/plain", mime.to_string());
}

#[test]
fn reference() {
    /*
     */

    let mime: Mime;
    {
        let string = "text/plain".to_string();
        mime = Mime::new(&string).unwrap().into_owned();
    }
    mime.to_string();
}
