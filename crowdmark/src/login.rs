use crate::error::CrowdmarkError;
use regex::Regex;
use reqwest::Client;

/// Logs in to Crowdmark
///
/// # Errors
///
/// Returns `CrowdmarkError` if the request to Crowdmark fails.
pub async fn get_token(email: String, password: String) -> Result<String, CrowdmarkError> {
    let client = Client::builder().cookie_store(true).build()?;
    let res = client
        .get("https://app.crowdmark.com/student")
        .send()
        .await?;
    let html = res.text().await?;
    let re = Regex::new(r#"<meta\s+name="csrf-token"\s+content="([^"]+)""#)?;
    let csrf = match re.captures(&html) {
        Some(captures) => captures[1].to_string(),
        None => {
            return Err(CrowdmarkError::NotAuthenticated(
                "Missing CSRF Token".to_string(),
            ));
        }
    };
    let params = [
        ("authenticity_token", csrf),
        ("user[email]", email.clone()),
        ("user[password]", password.clone()),
        ("commit", "Sign in".to_string()),
    ];
    client
        .post("https://app.crowdmark.com/sign-in")
        .form(&params)
        .send()
        .await?;

    Ok(client
        .get("https://app.crowdmark.com/")
        .send()
        .await?
        .cookies()
        .find(|c| c.name() == "cm_session_id")
        .ok_or(CrowdmarkError::Login())?
        .value()
        .to_string())
}
