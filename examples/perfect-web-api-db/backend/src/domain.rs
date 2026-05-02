#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub email: String,
    pub active: bool,
}

impl Account {
    pub fn new(id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: AccountId(id.into()),
            email: email.into(),
            active: true,
        }
    }
}

