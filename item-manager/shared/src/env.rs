#[derive(Default, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Production,
}

pub fn which() -> Environment {
    #[cfg(debug_assertions)]
    let default_env = Environment::Development;
    #[cfg(not(debug_assertions))]
    let default_env = Environment::Production;

    match std::env::var("ENV") {
        Ok(env) => env.parse().unwrap_or(default_env),
        Err(_) => default_env,
    }
}
