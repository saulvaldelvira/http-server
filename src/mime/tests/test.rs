use rmime::Mime;

#[test]
fn test() {
    const TESTS: [(&str,&str); 2] = [
        ("video.mp4", "video/mp4"),
        ("audio.aac", "audio/aac"),
    ];
    for (name,expected) in TESTS {
        let mime = Mime::from_filename(name).unwrap();
        assert_eq!(mime.to_string(), expected);
    };
}
