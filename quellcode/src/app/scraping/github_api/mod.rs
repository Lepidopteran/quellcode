use log::{debug, trace};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::APP_ID;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

mod repo_contents;
pub use repo_contents::*;
