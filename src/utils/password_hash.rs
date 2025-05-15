use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use error_stack::{Result, ResultExt};

use crate::errors::Error;

pub fn compute_password_hash(password: &str) -> Result<String, Error> {
    let arg2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    );
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = arg2
        .hash_password(password.as_bytes(), &salt)
        .change_context_lazy(|| Error::Message("failed to compute password hash".to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password_hash(pass: &str, hashed_password: &str) -> bool {
    let arg2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    );
    let Ok(hash) = PasswordHash::new(hashed_password) else {
        return false;
    };
    arg2.verify_password(pass.as_bytes(), &hash).is_ok()
}
