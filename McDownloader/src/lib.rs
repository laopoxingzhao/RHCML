const mc_vsersions_json :&str= "https://piston-meta.mojang.com/mc/game/version_manifest.json";

pub async fn get_versions() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.get(mc_vsersions_json).send().await?;
    let text = response.text().await?;
    Ok(text)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    #[tokio::test]
    async fn it_works() {
        let Ok(text) = get_versions().await else {
            println!("Error");
            return;
        };
    
        let json: HashMap<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
        let latest = json["latest"]["release"].as_str();
        let version_manifest = json["versions"].as_array().unwrap();
    
        // 使用 `filter` 和 `collect` 收集所有 "release" 版本
        let releases: Vec<_> = version_manifest.iter()
            .filter(|v| v["type"].as_str() == Some("release"))
            .collect();
    
        // 打印所有 release 版本
        for release in releases {
            println!("Release version: {:?}", release);
        }
    
        // 打印最新版本
        println!("Latest version: {}", latest.unwrap());
    }
  
}
