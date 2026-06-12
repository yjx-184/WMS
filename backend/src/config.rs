pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Reads `.env` file if present, then falls back to system environment.
    ///
    /// # Panics
    ///
    /// Panics if `DATABASE_URL` is not set or `SERVER_PORT` is not a valid u16.
    pub fn from_env() -> Self {
        // Load .env from the backend directory.
        // First try cwd (works when running from backend/),
        // then try relative to CARGO_MANIFEST_DIR (works with --manifest-path).
        let _ = dotenvy::dotenv();
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let _ = dotenvy::from_filename(
                std::path::Path::new(&manifest_dir).join(".env"),
            );
        }

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env or environment");

        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid u16");

        Config {
            database_url,
            server_port,
        }
    }

    /// Returns the database URL with the password masked for safe logging.
    pub fn masked_database_url(&self) -> String {
        mask_password(&self.database_url)
    }
}

/// Replace `:password@` with `:***@` in a connection URL.
///
/// Example: `postgres://user:secret@host/db` → `postgres://user:***@host/db`
fn mask_password(url: &str) -> String {
    // Locate the userinfo segment: between :// and the last @
    if let Some(scheme_end) = url.find("://") {
        let after_scheme = &url[scheme_end + 3..];
        if let Some(at_pos) = after_scheme.rfind('@') {
            let userinfo = &after_scheme[..at_pos];
            if let Some(colon_pos) = userinfo.rfind(':') {
                let prefix = &url[..scheme_end + 3 + colon_pos];
                let suffix = &url[scheme_end + 3 + at_pos..];
                return format!("{}:***{}", prefix, suffix);
            }
        }
    }
    // No password segment found — return as-is
    url.to_string()
}
