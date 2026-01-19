use reqwest::Client;
use serde_json::json;

// Check if the Ollama is already loaded
pub async fn is_model_loaded(base_url: &str, model: &str) -> anyhow::Result<bool> {
    let client = Client::new();
    let url = format!("{}/api/ps", base_url);
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Ok(false);
    }

    let json_resp: serde_json::Value = resp.json().await?;
    if let Some(models) = json_resp.get("models").and_then(|m| m.as_array()) {
        for m in models {
            if m.get("name").and_then(|n| n.as_str()) == Some(model) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

// Warm up Ollama model so it stays loaded
pub async fn warmup_ollama(base_url: &str, model: &str, keep_alive: i32) -> anyhow::Result<()> {
    // only run warm-up if not loaded
    if is_model_loaded(base_url, model).await? {
        println!(
            "Ollama model '{}' is already loaded, skipping warmup",
            model
        );
        return Ok(());
    }

    let client = Client::new();
    // let url = format!("{}/api/chat", base_url);
    let url = base_url;
    println!("url : {}", url);
    let resp = client
        .post(url)
        .json(&json!({
            "model": model,
            "messages": [
                { "role": "user", "content": "warm up" }
            ],
            "keep_alive": keep_alive
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await?;
        eprint!("Ollama warmpup failed: {}", body);
    } else {
        println!("Ollama model '{}' warmed up", model);
    }
    Ok(())
}
