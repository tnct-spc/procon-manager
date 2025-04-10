pub struct AppConfig {
    pub auth: AuthConfig,
}

impl AppConfig {
    pub fn new(secrets: shuttle_runtime::SecretStore) -> anyhow::Result<Self> {
        let auth = AuthConfig {
            ttl: secrets.get("AUTH_TOKEN_TTL").unwrap().parse()?,
            secret: secrets.get("JWT_SECRET").unwrap(),
        };
        Ok(Self { auth })
    }
}

pub struct AuthConfig {
    pub ttl: u64,
    pub secret: String,
}
