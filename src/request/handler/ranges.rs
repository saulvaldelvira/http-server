use std::ops::Range;

use crate::{server::{error::err, ServerError}, Result};

pub fn get_range_for(range: &str, len: u64) -> Result<Range<u64>> {
    let mut range = range.split("=");
    let unit = range.next().ok_or_else(|| ServerError::from_str("Missing unit"))?;
    if unit != "bytes" {
        /* TODO */
        return err!("Unknown unit").err();
    }
    /* TODO: Implement more ranges
     * -1024 (get the last 1024) */
    let range = range.next().ok_or_else(|| err!("Missing range"))?;
    let mut range = range.split("-");
    let start = range.next().unwrap_or("").parse()?;
    let end = range.next().unwrap_or("").parse().unwrap_or(len);
    Ok(start..end)
}

#[cfg(test)]
mod test {
    use super::get_range_for;

    #[test]
    fn unbounded() {
        let range = get_range_for("bytes=0-", 1024).unwrap();
        assert_eq!(range, 0..1024);
        let range = get_range_for("bytes=12", 1024).unwrap();
        assert_eq!(range, 12..1024);
    }

    #[test]
    fn bounded() {
        let range = get_range_for("bytes=40-70", 1024).unwrap();
        assert_eq!(range, 40..70);
    }

    #[test]
    fn no_unit() {
        match get_range_for("=0-", 1024) {
            Ok(_) => panic!(),
            Err(err) => assert_eq!("Unknown unit", err.get_message()),
        }
    }

    #[test]
    fn no_range() {
        match get_range_for("bytes=", 1024) {
            Ok(_) => panic!(),
            Err(err) => assert_eq!("cannot parse integer from empty string", err.get_message()),
        }
    }
}
