use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // user_uid
    pub roles: Vec<String>,
    pub org_id: Option<i64>,
    pub plan: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl: Duration::hours(2),
            refresh_ttl: Duration::days(30),
        }
    }

    pub fn issue_access_token(&self, claims: &Claims) -> anyhow::Result<String> {
        let mut claims = claims.clone();
        let now = Utc::now();
        claims.iat = now.timestamp() as usize;
        claims.exp = (now + self.access_ttl).timestamp() as usize;
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn issue_refresh_token(&self, user_uid: &str) -> anyhow::Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_uid.to_string(),
            roles: vec![],
            org_id: None,
            plan: String::new(),
            iat: now.timestamp() as usize,
            exp: (now + self.refresh_ttl).timestamp() as usize,
        };
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn verify_access_token(&self, token: &str) -> anyhow::Result<Claims> {
        let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(data.claims)
    }
}
