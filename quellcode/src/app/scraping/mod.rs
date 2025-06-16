pub mod github_api;
pub mod package_control;

#[cfg(test)]
pub use super::tests::*;

#[cfg(test)]
mod test_util {
    pub use super::init;

    pub fn github_token() -> Option<secrecy::SecretString> {
        init();
        if std::env::var("CI").is_ok() {
            return None;
        }

        crate::app::github_token().unwrap()
    }
}
