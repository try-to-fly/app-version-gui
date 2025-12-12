use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct HomebrewFormula {
    versions: HomebrewVersions,
}

#[derive(Deserialize)]
struct HomebrewVersions {
    stable: String,
}

pub async fn get_version(formula: &str) -> Result<String, String> {
    let client = Client::new();
    let url = format!("https://formulae.brew.sh/api/formula/{}.json", formula);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Homebrew API error: {}", response.status()));
    }

    let formula_info: HomebrewFormula = response.json().await.map_err(|e| e.to_string())?;

    Ok(formula_info.versions.stable)
}
