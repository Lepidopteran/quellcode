use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "type")]
    pub type_: String,
    pub size: i64,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub _links: Links,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub git: Option<String>,
    pub html: Option<String>,
    #[serde(rename = "self")]
    pub self_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_: String,
    pub size: i64,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub content: Option<String>,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub entries: Option<Vec<Entry>>,
    pub encoding: Option<String>,
    pub _links: Links,
}

impl Content {
    pub fn is_dir(&self) -> bool {
        self.type_ == "dir"
    }

    pub fn is_file(&self) -> bool {
        self.type_ == "file"
    }
}

#[derive(Debug)]
pub enum ContentResponse {
    Dir(Vec<Content>),
    File(Content),
}

impl From<Content> for ContentResponse {
    fn from(content: Content) -> Self {
        ContentResponse::File(content)
    }
}

impl From<Vec<Content>> for ContentResponse {
    fn from(contents: Vec<Content>) -> Self {
        ContentResponse::Dir(contents)
    }
}

impl ContentResponse {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Dir(_) => true,
            Self::File(_) => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Dir(_) => false,
            Self::File(_) => true,
        }
    }
}

pub trait RepoContents {
    async fn get_content(&self, owner: &str, repo: &str, path: &str) -> Result<ContentResponse>;
    async fn get_content_from_url(&self, url: &str) -> Result<ContentResponse>;
}

impl RepoContents for GithubApi {
    async fn get_content(&self, owner: &str, repo: &str, path: &str) -> Result<ContentResponse> {
        get_content(
            &self.client,
            self.token.as_ref().map(|t| t.expose_secret()),
            owner,
            repo,
            path,
        )
        .await
    }

    async fn get_content_from_url(&self, url: &str) -> Result<ContentResponse> {
        get_content_from_url(
            &self.client,
            self.token.as_ref().map(|t| t.expose_secret()),
            url,
        )
        .await
    }
}

pub async fn get_content(
    client: &reqwest::Client,
    token: Option<&str>,
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<ContentResponse> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");

    let response = if let Some(token) = token {
        client.get(&url).bearer_auth(token)
    } else {
        client.get(&url)
    }
    .header(USER_AGENT, APP_ID)
    .header(ACCEPT, ACCEPT_VALUE)
    .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
    .send()
    .await?;

    trace!("Github API Response: {:#?}", response);
    let bytes = response.bytes().await?;
    let content_response: ContentResponse = {
        if let Ok(content) = serde_json::from_slice::<Content>(&bytes) {
            content.into()
        } else {
            let contents = serde_json::from_slice::<Vec<Content>>(&bytes)?;

            contents.into()
        }
    };

    Ok(content_response)
}

pub async fn get_content_from_url(
    client: &reqwest::Client,
    token: Option<&str>,
    url: &str,
) -> Result<ContentResponse> {
    let parts: Vec<_> = url.split('/').collect();
    if parts.len() < 5 || !url.contains("github.com") {
        return Err(GithubApiError::InvalidUrl);
    }

    let path = parts
        .iter()
        .skip(5)
        .copied()
        .collect::<Vec<&str>>()
        .join("/");

    debug!(
        "Getting content from Owner: {}, Repo: {}, Path: {}",
        parts[3], parts[4], path
    );

    get_content(client, token, parts[3], parts[4], &path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = dotenvy::dotenv();
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_get_content() {
        init();
        let client = reqwest::Client::new();
        let content_tree = get_content(
            &client,
            std::env::var("QUELLCODE_GITHUB_TOKEN").ok().as_deref(),
            "octocat",
            "hello-world",
            "README",
        )
        .await;

        debug!("content_tree: {:#?}", content_tree);
        assert!(content_tree.is_ok());
    }

    #[tokio::test]
    async fn test_get_content_from_url() {
        init();
        let client = reqwest::Client::new();
        for url in [
            "https://github.com/microsoft/vscode",
            "https://github.com/microsoft/vscode/README.md",
        ]
        .iter()
        {
            let content = get_content_from_url(
                &client,
                std::env::var("QUELLCODE_GITHUB_TOKEN").ok().as_deref(),
                url,
            )
            .await;
            debug!("content_tree: {:#?}", content);
            assert!(content.is_ok());
        }
    }
}
