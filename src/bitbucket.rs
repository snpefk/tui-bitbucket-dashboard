use data_encoding::BASE64;

pub struct BitBucket<'a> {
    pub auth_header: (&'a str, String),
    pub project_url: String,
}

impl<'a> BitBucket<'a> {
    const PR_PATH: &'a str = "/{repo}/pull-requests";

    pub fn new(user: &str, password: &str, host: &str, project: &str) -> Self {
        let credentials = format!("{}:{}", user, password);
        let base64 = BASE64.encode(credentials.as_bytes());

        let project_url = format!(
            "https://{host}/rest/api/1.0/projects/{project}/repos",
            host = host,
            project = project
        );

        Self {
            auth_header: ("Authorization", base64),
            project_url,
        }
    }

    pub async fn request_repos(self) {
        todo!("not implemented");
    }

    pub async fn request_pr_data(self) {
        todo!("not implemented")
    }

    async fn get_next_page(self, current_url: &str, current_page: usize) {
        todo!("not implemented")
    }
}
