pub struct StrSplit<'a> {
    remainder: Option<&'a str>,
    delimiter: &'a str,
}

impl<'a> StrSplit<'a> {
    //Strsplit can only be used for aslong as the haystack and delimiter is live
    pub fn new(haystack: &'a str, delimiter: &'a str) -> Self {
        Self {
            remainder: Some(haystack),
            delimiter,
        }
    }
}

impl<'a> Iterator for StrSplit<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.as_mut()?;
        if let Some(next_delim) = remainder.find(self.delimiter) {
            let until_delim = &remainder[..next_delim];
            *remainder = &remainder[(next_delim + self.delimiter.len())..];
            Some(until_delim)
        } else {
            self.remainder.take()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn working_example() {
        let haystack = "a b c d e";
        let letters_collected: Vec<&str> = StrSplit::new(haystack, " ").collect();
        assert_eq!(letters_collected, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn tail_example() {
        let haystack = "a b c d ";
        let letters_collected: Vec<&str> = StrSplit::new(haystack, " ").collect();
        assert_eq!(letters_collected, vec!["a", "b", "c", "d", ""]);
    }
}
