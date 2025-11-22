use crowdmark::error::CrowdmarkError;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;

#[derive(Deserialize, Serialize)]
pub struct LoginDetails {
    email: String,
    password: String,
}

pub async fn get_token() -> Result<String, CrowdmarkError> {
    let entry =
        Entry::new("climark", &whoami::username()).expect("Couldn't create keyring entry: {err}");
    let details = if let Ok(password) = entry.get_password() {
        let details: LoginDetails =
            serde_json::from_str(&password).expect("Failed to decode keyring JSON");
        details
    } else {
        let details = LoginDetails {
            email: get_email(),
            password: get_password(),
        };

        entry
            .set_password(&serde_json::to_string(&details).expect("Failed to encode keyring JSON"))
            .expect("Failed to set keyring password");
        details
    };

    crowdmark::login::get_token(details.email, details.password).await
}

fn get_email() -> String {
    print!("Please enter your email: ");
    io::stdout().flush().unwrap();

    let mut email = String::new();
    io::stdin()
        .read_line(&mut email)
        .expect("Failed to read line");
    email.trim().to_string()
}

fn get_password() -> String {
    print!("Please enter your password: ");
    io::stdout().flush().unwrap();
    rpassword::read_password().expect("Failed to read password")
}
