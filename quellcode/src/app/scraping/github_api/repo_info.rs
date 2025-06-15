use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub html_url: Url,
    pub full_name: String,
    pub default_branch: String,
    pub owner: Owner,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Owner {
    pub login: String,
    pub avatar_url: Url,
}

pub trait RepoInfo {
    async fn get_repo_info(&self, owner: &str, repo: &str) -> Result<Repo, GithubApiError>;
    async fn get_repo_info_from_url(&self, url: &Url) -> Result<Repo, GithubApiError>;
}

impl RepoInfo for GithubApi {
    async fn get_repo_info(&self, owner: &str, repo: &str) -> Result<Repo, GithubApiError> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");

        let response = if let Some(token) = self.token.as_ref().map(|s| s.expose_secret()) {
            self.client.get(&url).bearer_auth(token)
        } else {
            self.client.get(&url)
        }
        .header(USER_AGENT, APP_ID)
        .header(ACCEPT, JSON_MEDIA_TYPE)
        .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
        .send()
        .await?;

        trace!("Github API Response: {:#?}", response);

        Ok(response.json().await?)
    }

    async fn get_repo_info_from_url(&self, url: &Url) -> Result<Repo, GithubApiError> {
        let path = url.path().trim_start_matches('/').to_string();
        let parts: Vec<_> = path.split('/').collect();

        if parts.len() < 2 || !url.host_str().unwrap().contains("github.com") {
            return Err(GithubApiError::InvalidUrl);
        }

        self.get_repo_info(parts[0], parts[1]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = dotenvy::dotenv();
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_get_repo_info() {
        init();

        let api = GithubApi::new(reqwest::Client::new(), github_token());
        let response = api.get_repo_info("octocat", "Hello-World").await.unwrap();

        debug!("Github Repo Info: {:#?}", response);
        assert_eq!(response.name, "Hello-World");
    }

    #[tokio::test]
    async fn test_get_repo_info_from_url() {
        init();

        let api = GithubApi::new(reqwest::Client::new(), github_token());
        let response = api
            .get_repo_info_from_url(&Url::parse("https://github.com/octocat/Hello-World").unwrap())
            .await
            .unwrap();

        debug!("Github Repo Info: {:#?}", response);
        assert_eq!(response.name, "Hello-World");
    }
}
