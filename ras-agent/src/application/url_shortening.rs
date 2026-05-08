use indexmap::IndexMap;
use regex::Regex;

const MIN_URL_LEN: usize = 80;

#[derive(Debug, Default)]
pub struct UrlShortener {
    map: IndexMap<String, String>,
}

impl UrlShortener {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shorten(&mut self, text: &str) -> String {
        let Ok(re) = Regex::new(r#"https?://[^\s"'<>]+"#) else {
            return text.to_string();
        };
        let mut next_id = self.map.len();
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;
        for m in re.find_iter(text) {
            result.push_str(&text[last_end..m.start()]);
            let original = m.as_str();
            if original.len() < MIN_URL_LEN {
                result.push_str(original);
            } else if let Some(short) = self.lookup_existing(original) {
                result.push_str(&short);
            } else {
                let key = format!("ras://url/{next_id}");
                self.map.insert(key.clone(), original.to_string());
                result.push_str(&key);
                next_id += 1;
            }
            last_end = m.end();
        }
        result.push_str(&text[last_end..]);
        result
    }

    pub fn restore(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (short, full) in &self.map {
            out = out.replace(short, full);
        }
        out
    }

    fn lookup_existing(&self, original: &str) -> Option<String> {
        self.map.iter().find_map(|(k, v)| (v == original).then(|| k.clone()))
    }
}
