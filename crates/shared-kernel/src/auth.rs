use serde::{Deserialize, Serialize};

use crate::ids::UserId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: UserId,
    pub role: String,
    pub exp: i64,
}
