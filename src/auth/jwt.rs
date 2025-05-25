use error_stack::{Result, ResultExt};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::errors::Error;

const JWT_ALGORITHM: Algorithm = Algorithm::HS512;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: i64,
    exp: u64,
    #[serde(default, flatten)]
    pub claims: Map<String, Value>,
}

#[derive(Debug)]
pub struct JWT {
    secret: String,
    algorithm: Algorithm,
}

impl JWT {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
            algorithm: JWT_ALGORITHM,
        }
    }

    pub fn algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn generate_token(
        &self,
        expiration: u64,
        user_id: i64,
        claims: Map<String, Value>,
    ) -> Result<String, Error> {
        let exp = get_current_timestamp().saturating_add(expiration);

        let claims = UserClaims {
            user_id,
            exp,
            claims,
        };

        let token = encode(
            &Header::new(self.algorithm),
            &claims,
            &EncodingKey::from_base64_secret(&self.secret)
                .change_context_lazy(|| Error::Message("failed to encode token".to_string()))?,
        )
        .change_context_lazy(|| Error::Message("failed to encode token".to_string()))?;

        Ok(token)
    }

    pub fn validate(&self, token: &str) -> Result<TokenData<UserClaims>, Error> {
        let mut validate = Validation::new(self.algorithm);
        validate.leeway = 0;

        let token_data = decode::<UserClaims>(
            token,
            &DecodingKey::from_base64_secret(&self.secret)
                .change_context_lazy(|| Error::Message("failed to decode token".to_string()))?,
            &validate,
        )
        .change_context_lazy(|| Error::Message("failed to decode token".to_string()))?;

        Ok(token_data)
    }
}
