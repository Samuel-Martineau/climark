use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;

#[derive(Deserialize, Serialize)]
pub struct LoginDetails {
    email: String,
    password: String,
}

pub fn get_login() -> LoginDetails {
    let entry = match Entry::new("climark", &whoami::username()) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("Couldn't create keyring entry: {err}");
            std::process::exit(1)
        }
    };
    let details = match entry.get_password() {
        Ok(password) => {
            let details: LoginDetails =
                serde_json::from_str(&password).expect("Failed to decode keyring JSON");
            details
        }
        Err(_) => {
            let details = LoginDetails {
                email: get_email(),
                password: get_password(),
            };

            entry
                .set_password(
                    &serde_json::to_string(&details).expect("Failed to encode keyring JSON"),
                )
                .expect("Failed to set keyring password");
            details
        }
    };

    details
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
