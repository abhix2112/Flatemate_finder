// src/utils/twilio.rs
use std::env;
use reqwest::Client;

pub async fn send_otp_via_twilio_verify(phone: &str) -> Result<(), String> {
    let account_sid = env::var("TWILIO_ACCOUNT_SID").map_err(|e| e.to_string())?;
    let auth_token = env::var("TWILIO_AUTH_TOKEN").map_err(|e| e.to_string())?;
    let verify_sid = env::var("TWILIO_VERIFY_SERVICE_SID").map_err(|e| e.to_string())?;

    let client = Client::new();
    let url = format!("https://verify.twilio.com/v2/Services/{}/Verifications", verify_sid);

    let res = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[
            ("To", phone),
            ("Channel", "sms"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown error".to_string()))
    }
}

pub async fn verify_otp_via_twilio(phone: &str, code: &str) -> Result<bool, String> {
    let account_sid = env::var("TWILIO_ACCOUNT_SID").map_err(|e| e.to_string())?;
    let auth_token = env::var("TWILIO_AUTH_TOKEN").map_err(|e| e.to_string())?;
    let verify_sid = env::var("TWILIO_VERIFY_SERVICE_SID").map_err(|e| e.to_string())?;

    let client = Client::new();
    let url = format!("https://verify.twilio.com/v2/Services/{}/VerificationCheck", verify_sid);

    let res = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[
            ("To", phone),
            ("Code", code),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let body = res.text().await.map_err(|e| e.to_string())?;
    
    if status.is_success() && body.contains("\"status\": \"approved\"") {
      Ok(true)
    } else {
        Ok(false)
    }
}
