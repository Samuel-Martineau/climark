use crate::error::CrowdmarkError;
use reqwest::Client;

/// Logs in to Crowdmark
///
/// # Errors
///
/// Returns [`CrowdmarkError`] if the request to Crowdmark fails.
pub async fn get_token(email: String, password: String) -> Result<String, CrowdmarkError> {
    let client = Client::builder().cookie_store(true).build()?;
    let csrf = crate::get_csrf(None).await?;
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
