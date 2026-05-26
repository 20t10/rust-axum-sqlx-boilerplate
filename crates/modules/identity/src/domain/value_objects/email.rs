#[derive(Clone, Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        if value.contains('@') {
            Ok(Self(value.to_owned()))
        } else {
            Err("invalid email")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
