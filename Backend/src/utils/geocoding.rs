use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    display_name: String,
}

pub async fn reverse_geocode(lat: f64, lon: f64) -> Option<String> {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
        lat, lon
    );

    let res = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "flatmate-finder-app/1.0")
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        let body: NominatimResponse = res.json().await.ok()?;
        Some(body.display_name)
    } else {
        None
    }
}
