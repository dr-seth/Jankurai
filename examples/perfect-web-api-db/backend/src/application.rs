use crate::domain::Account;

pub fn can_view_admin_dashboard(account: &Account) -> bool {
    account.active && account.email.ends_with("@example.com")
}

