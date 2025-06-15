use reqwest::header::CONTENT_TYPE;

use super::*;

pub trait GitBlobText {
    async fn get_file_text(&self, owner: &str, repo: &str, blob_sha: &str) -> Result<String>;
}

impl GitBlobText for GithubApi {
    async fn get_file_text(&self, owner: &str, repo: &str, blob_sha: &str) -> Result<String> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/git/blobs/{blob_sha}");

        let response = if let Some(token) = self.token.as_ref().map(|s| s.expose_secret()) {
            self.client.get(&url).bearer_auth(token)
        } else {
            self.client.get(&url)
        }
        .header(USER_AGENT, APP_ID)
        .header(ACCEPT, RAW_MEDIA_TYPE)
        .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
        .send()
        .await?;

        trace!("Github API Response: {:#?}", response);

        if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
            if !content_type.to_str().unwrap().contains("text/plain") {
                return Err(GithubApiError::InvalidResponse);
            }
        } else {
            return Err(GithubApiError::InvalidResponse);
        }

        Ok(response.text().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        init();

        let api = GithubApi::new(reqwest::Client::new(), github_token());
        let response = api
            .get_file_text(
                "octocat",
                "hello-world",
                "980a0d5f19a64b4b30a87d4206aade58726b60e3",
            )
            .await
            .unwrap();

        assert!(!response.is_empty());
    }
}
