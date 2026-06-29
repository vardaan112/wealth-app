pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub app_user_email: Option<String>,
    pub app_user_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://wealth_user:wealth_password@localhost:5432/wealth_app".to_string()
        });

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8000);

        let jwt_secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-change-me-jwt-secret".to_string());
        let app_user_email = std::env::var("APP_USER_EMAIL").ok();
        let app_user_password = std::env::var("APP_USER_PASSWORD").ok();

        Self {
            database_url,
            port,
            jwt_secret,
            app_user_email,
            app_user_password,
        }
    }
}
