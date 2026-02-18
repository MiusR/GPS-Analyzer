use std::env;


#[derive(Clone, Debug)]
pub struct Config {
    server_host: String,
    server_port: u16,
    app_url: String,

    // JWT
    jwt_access_secret: String,
    jwt_refresh_secret: String,

    // Google OAuth2
    google_client_id: String,
    google_client_secret: String,

    // GitHub OAuth2
    github_client_id: String,
    github_client_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),

            jwt_access_secret: env::var("JWT_ACCESS_SECRET")
                .expect("JWT_ACCESS_SECRET must be set"),
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .expect("JWT_REFRESH_SECRET must be set"),

            google_client_id: env::var("GOOGLE_CLIENT_ID")
                .expect("GOOGLE_CLIENT_ID must be set"),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),

            github_client_id: env::var("GITHUB_CLIENT_ID")
                .expect("GITHUB_CLIENT_ID must be set"),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
        }
    }

    pub fn google_redirect_uri(&self) -> String {
        format!("{}/auth/google/callback", self.app_url)
    }

    pub fn github_redirect_uri(&self) -> String {
        format!("{}/auth/github/callback", self.app_url)
    }

    pub fn get_server_host(&self) -> &str {
        &self.server_host
    }

    pub fn get_server_port(&self) -> u16 {
        self.server_port
    }

    pub fn get_app_url(&self) -> &str {
        &self.app_url
    }

    pub fn get_jwt_access_secret(&self) -> &str {
        &self.jwt_access_secret
    }
    
    pub fn get_jwt_refresh_secret(&self) -> &str {
        &self.jwt_refresh_secret
    }

    pub fn get_google_client_id(&self) -> &str {
        &self.google_client_id
    }

    pub fn get_google_client_secret(&self) -> &str {
        &self.google_client_secret
    }

    pub fn get_github_client_id(&self) -> &str {
        &self.github_client_id
    }

    pub fn get_github_client_secret(&self) -> &str {
        &self.github_client_secret
    }
}
