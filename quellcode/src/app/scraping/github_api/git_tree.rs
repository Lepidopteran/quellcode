use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub sha: String,
    pub url: Url,
    pub truncated: bool,
    pub tree: Vec<TreeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeItem {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub sha: String,
    pub size: Option<u64>,
    pub url: Option<Url>,
}

impl TreeItem {
    pub fn is_dir(&self) -> bool {
        self.type_ == "tree"
    }

    pub fn is_file(&self) -> bool {
        self.type_ == "blob"
    }
}

pub trait GitTree {
    async fn get_tree(&self, owner: &str, repo: &str, branch: &str) -> Result<Tree>;
    async fn get_tree_from_url(&self, url: &str, guess_branch_if_missing: bool) -> Result<Tree>;
}

impl GitTree for GithubApi {
    async fn get_tree(&self, owner: &str, repo: &str, branch: &str) -> Result<Tree> {
        get_tree(
            &self.client,
            self.token.as_ref().map(|t| t.expose_secret()),
            owner,
            repo,
            branch,
        )
        .await
    }

    async fn get_tree_from_url(&self, url: &str, guess_branch_if_missing: bool) -> Result<Tree> {
        get_tree_from_url(
            &self.client,
            self.token.as_ref().map(|t| t.expose_secret()),
            url,
            guess_branch_if_missing,
        )
        .await
    }
}

pub async fn get_tree(
    client: &reqwest::Client,
    token: Option<&str>,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Tree> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/git/trees/{branch}?recursive=1");
    let response = if let Some(token) = token {
        client.get(&url).bearer_auth(token)
    } else {
        client.get(&url)
    }
    .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
    .header(ACCEPT, JSON_MEDIA_TYPE)
    .header(USER_AGENT, APP_ID)
    .send()
    .await?;

    trace!("Github API Response: {:#?}", response);

    Ok(response.json().await?)
}

pub async fn get_tree_from_url(
    client: &reqwest::Client,
    token: Option<&str>,
    url: &str,
    guess_branch_if_missing: bool,
) -> Result<Tree> {
    let url = Url::parse(url)?;

    if !url
        .host_str()
        .is_some_and(|host| host.contains("github.com"))
        || url.path().is_empty()
    {
        return Err(GithubApiError::InvalidUrl);
    }

    trace!("Parsed URL: {:#?}", url);

    if let Some(segments) = url.path_segments().map(|c| c.collect::<Vec<_>>()) {
        trace!("Got Segments: {:#?}", segments);
        let (owner, repo, branch) = match segments.len() {
            2 => (segments[0], segments[1], None),
            4 => (segments[0], segments[1], Some(segments[3])),
            _ => return Err(GithubApiError::InvalidUrl),
        };
        if let Some(branch) = branch {
            get_tree(client, token, owner, repo, branch).await
        } else if guess_branch_if_missing {
            for branch in ["master", "main", "develop"] {
                if let Ok(tree) = get_tree(client, token, owner, repo, branch).await {
                    return Ok(tree);
                }
            }

            Err(GithubApiError::InvalidUrl)
        } else {
            Err(GithubApiError::InvalidUrl)
        }
    } else {
        Err(GithubApiError::InvalidUrl)
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
    async fn test_get_tree() {
        init();

        let client = reqwest::Client::new();
        let tree = get_tree(
            &client,
            std::env::var("GITHUB_TOKEN").ok().as_deref(),
            "octocat",
            "hello-world",
            "master",
        )
        .await;
        debug!("tree: {:#?}", tree);
        assert!(tree.is_ok());
    }

    #[tokio::test]
    async fn test_get_tree_from_url() {
        init();
        let client = reqwest::Client::new();
        let tree = get_tree_from_url(
            &client,
            std::env::var("GITHUB_TOKEN").ok().as_deref(),
            "https://github.com/octocat/hello-world/tree/master",
            true,
        )
        .await;

        debug!("tree: {:#?}", tree);
        assert!(tree.is_ok());
    }
}
