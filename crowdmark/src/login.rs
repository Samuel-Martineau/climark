use crate::error::CrowdmarkError;
use regex_lite::Regex;
use reqwest::Client;

/// Logs in to Crowdmark
///
/// # Errors
///
/// Returns [`CrowdmarkError`] if the request to Crowdmark fails.
pub async fn get_token(email: String, password: String) -> Result<String, CrowdmarkError> {
    let client = Client::builder().cookie_store(true).build()?;
    let resp = client
        .get("https://app.crowdmark.com/sign-in")
        .send()
        .await?;

    let re = Regex::new(r#"name="authenticity_token" value="([^"]+)""#)?;
    let authenticity_token = match re.captures(&resp.text().await?) {
        Some(captures) => captures[1].to_string(),
        None => {
            return Err(CrowdmarkError::NotAuthenticated(
                "Missing authenticity token".to_string(),
            ));
        }
    };
    let params = [
        ("authenticity_token", authenticity_token),
        ("user[email]", email.clone()),
        ("user[password]", password.clone()),
        ("commit", "Sign+in".to_string()),
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
