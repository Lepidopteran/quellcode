pub mod github_api;
pub mod package_control;

const APP_ID: &str = concat!(env!("CARGO_PKG_NAME"), ", V", env!("CARGO_PKG_VERSION"));

#[cfg(test)]
mod test_util {

    pub fn github_token() -> Option<secrecy::SecretString> {
        if std::env::var("CI").is_ok() {
            return None;
        }

        crate::github_token().unwrap()
    }
}
