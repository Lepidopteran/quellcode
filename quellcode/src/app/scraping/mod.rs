pub mod github_api;
pub mod package_control;

#[cfg(test)]
mod test_util {
    pub fn init() {
        dotenvy::dotenv().ok();
        let _ = env_logger::builder().is_test(true).try_init();
        let _ = color_eyre::install();
    }

    pub fn github_token() -> Option<secrecy::SecretString> {
        if std::env::var("CI").is_ok() {
            return None;
        }

        crate::app::github_token().unwrap()
    }
}
