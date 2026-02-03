use crate::error::CrowdmarkError;
use regex_lite::Regex;
use reqwest::Client;

/// Logs in to Crowdmark
///
/// # Errors
///
/// Returns [`CrowdmarkError`] if the request to Crowdmark fails.
#[inline]
pub async fn get_token(email: String, password: String) -> Result<String, CrowdmarkError> {
    let client = Client::builder().cookie_store(true).build()?;
    let resp = client
        .get("https://app.crowdmark.com/sign-in")
        .send()
        .await?;

    let re = Regex::new(r#"name="authenticity_token" value="([^"]+)""#)?;
    let authenticity_token = re
        .captures(&resp.text().await?)
        .map(|capture| capture[1].to_string())
        .ok_or_else(|| CrowdmarkError::NotAuthenticated("Missing authenticity token".into()))?;
    let params = [
        ("authenticity_token", authenticity_token),
        ("user[email]", email.to_owned()),
        ("user[password]", password.to_owned()),
        ("commit", "Sign+in".to_owned()),
    ];

    let login_resp = client
        .post("https://app.crowdmark.com/sign-in")
        .form(&params)
        .send()
        .await?;

    Ok(login_resp
        .cookies()
        .find(|cookie| cookie.name() == "cm_session_id")
        .ok_or(CrowdmarkError::Login())?
        .value()
        .to_owned())
}
