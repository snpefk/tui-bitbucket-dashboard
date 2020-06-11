use data_encoding::BASE64;

pub struct BitBucket<'a> {
    pub auth_header: (&'a str, String),
}

impl<'a> BitBucket<'a> {
    pub fn new(user: &str, password: &str) -> Self {
        let credentials = format!("{}:{}", user, password);
        let base64 = BASE64.encode(credentials.as_bytes());

        Self {
            auth_header: ("Authorization", base64),
        }
    }
}
