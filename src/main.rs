use std::{
    collections::HashMap,
    io::Write,
    iter::{Map, Product},
    process::Stdio,
};

use axum::{
    Error, Router,
    extract::{Path, Query},
    routing::get,
};
use reqwest::Client;
use serde_json::map;
use once_cell::sync::Lazy;
// use serde::Deserialize;
use tokio::{
    net::{TcpListener, TcpSocket},
    process::Command,
};

const OAUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf?\
        client_id=00fa7ee8-8469-45e4-a4d0-f4b43a4127aa&\
        scope=XboxLive.signin offline_access&\
        redirect_uri=http://localhost:9999&\
        response_type=code&\
        response_mode=query";
        
 const CONFIG: &str = include_str!("config.txt");
 
static CLIENT_ID: Lazy<String> = Lazy::new(|| {
    serde_json::from_str::<serde_json::Value>(CONFIG).unwrap()["client_id"].as_str().unwrap().to_string()
});
static CLIENT_SECRET: Lazy<String> = Lazy::new(|| {
    serde_json::from_str::<serde_json::Value>(CONFIG).unwrap()["client_secret"].as_str().unwrap().to_string()
});
//Tenant  ID    80a19751-402c-4d77-8e9c-351c536849d8
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client_id: &str = serde_json::from_str::<serde_json::Value>(CONFIG).unwrap()["client_id"].as_str().unwrap();
    let client_secret: &str = serde_json::from_str::<serde_json::Value>(CONFIG).unwrap()["client_secret"].as_str().unwrap();

    let listener = TcpListener::bind("127.0.0.1:9999").await?;
    println!("Listening on http://127.0.0.1:9999");

    let router = Router::new().route("/", get(handler))
    // .route("/{code}", get(handler2))
    ;
    // const  OAUTH_URL: &str =  "https://login.live.com/oauth20_authorize.srf?\
    //     client_id=00fa7ee8-8469-45e4-a4d0-f4b43a4127aa&\
    //     scope=XboxLive.signin offline_access&\
    //     redirect_uri=http://localhost:9999&\
    //     response_type=code&\
    //     response_mode=query";
    tokio::spawn(async {
        if webbrowser::open(OAUTH_URL).is_ok() {
            println!("成功打开浏览器，请完成授权流程");
        } else {
            eprintln!("无法打开浏览器");
        }
    });
    axum::serve(listener, router).await?;
    Ok(())
}

async fn handler(Query(query): Query<HashMap<String, String>>) -> String {
    println!("Query: {:?}", query);
    let code = query.get("code");

    if let Some(code) = code {
        println!("Code: {}", code);
        // return format!("Hello, {}!", code);
        // let code = code.to_string();

        if let Ok(text) = token(code).await {
            // println!("Response: {}", text);
            let json: HashMap<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
            if let Some(RpsTicket) = json.get("access_token") {
                if let Some(rps_ticket_str) = RpsTicket.as_str() {
                    let rps = rps_ticket_str.trim_start_matches('"');
                    println!("RpsTicket: {}\n", rps);
                    let (token, uhs) = XboxLive(rps).await.unwrap();
                    println!("token: {}\n", token);
                    println!("uhs: {}\n", uhs);
                    // println!("Access Token: {:#?}", map);
                    let res = XSTS(token.as_str()).await.unwrap();
                    println!("XSTS: {}\n", res);

                    let res = Client::new()
                                            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
                                            .header("Content-Type", "application/json")
                                            .body(format!(
                                                r#"{{
                                                    "identityToken": "XBL3.0 x={uhs};{res}"
                                                }}"#
                                            ))
                                            .send()
                                            .await
                                            .unwrap();
                    let text = res.text().await.unwrap();
                    println!("Response: {}", text);



                    return text;
                }
            }

            return "text".to_string();
        }

        return "E, World!".to_string();
    } else {
        return "Hello, World!".to_string();
    }
    // return "Hello, World!".to_string();
}

/// 获取token
async fn token(code: &String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
                    .post("https://login.live.com/oauth20_token.srf")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(format!("client_id={}&code={}&grant_type=authorization_code&redirect_uri=http://localhost:9999&scope=XboxLive.signin offline_access&client_secret={}",*CLIENT_ID, code,*CLIENT_SECRET))
                    .send()
                    .await
                    .unwrap();
    let text = response.text().await;
    text
}
async fn resh_token(code: &String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
                    .post("https://login.live.com/oauth20_token.srf")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(format!("client_id={}&grant_type=refresh_token&scope=XboxLive.signin offline_access&client_secret={}",*CLIENT_ID,*CLIENT_SECRET))
                    .send()
                    .await
                    .unwrap();
    let text = response.text().await;
    text
}

async fn XboxLive(rpsTicket: &str) -> Result<(String, String), Error> {
    let client = reqwest::Client::new();
    // https://login.microsoftonline.com/consumers/oauth2/v2.0/token
    // https://user.auth.xboxlive.com/user/authenticate\
    let json2 = r#"{
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": "d=$()"
            },
             "TokenType": "JWT",
    "RelyingParty": "http://auth.xboxlive.com"
    }"#;
    let json3 = json2.replace("d=$()", &format!("d={}", rpsTicket));
    // println!("\n{}\n",json3);

    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .body(json3)
        .send()
        .await
        .unwrap();
    let text = response.text().await.unwrap();
    // println!("\n\nResponse: {}", text);
    let resp_json: serde_json::Value = serde_json::from_str(&text).unwrap();

    // 提取 Token
    let token = resp_json["Token"]
        .as_str()
        .ok_or("Failed to parse Token")
        .unwrap()
        .to_string();

    // 提取 uhs
    let uhs = resp_json["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("Failed to parse uhs")
        .unwrap()
        .to_string();

    Ok((token, uhs))
}

///XSTS身份验证
/// POST https://xsts.auth.xboxlive.com/xsts/authorize
async fn XSTS(token: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let body = r#"
        {
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [
                "<xbl_token>"
            ]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    }
    "#;
    let body = body.replace("<xbl_token>", token);

    println!("\n{}\n", body);

    let response = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
    let text = response.text().await.unwrap();
    Ok(text)
}


#[cfg(test)]
mod  tests {
    use std::{collections::HashMap, sync::Arc};

    use reqwest::Client;
    use tokio::{fs::File, io::AsyncWriteExt};

   #[tokio::test]
    async fn it_works() {
    
        let url = Arc::new(tokio::sync::Mutex::new("".to_string()));
        // let url = url.clone();
        let url_ = Arc::clone(&url); // 显式克隆Arc

       let handle = tokio::spawn(async move {
           let v= McDownloader::get_versions().await.unwrap();
        //    eprintln!("v: {:?}", v);
       let json: HashMap<String, serde_json::Value> = serde_json::from_str(&v).unwrap();
       let latest = json["latest"]["release"].as_str();
       let version_manifest = json["versions"].as_array().unwrap();

       // 使用 `filter` 和 `collect` 收集所有 "release" 版本
       let releases: Vec<_> = version_manifest.iter()
           .filter(|v| v["type"].as_str() == Some("release"))
           .collect();
        let  latest_url:String = releases.iter().find(|v| v["id"].as_str() == latest).unwrap().to_string();
        eprintln!("Latest version URL: {}", latest_url);
        url_.lock().await.push_str(&latest_url);
       });
       handle.await.unwrap(); // 等待任务完成

        let binding = serde_json::from_str::<serde_json::Value>(url.lock().await.as_str()).unwrap();
        let ur = binding.as_object().unwrap();
        
       
        let url_str = ur["url"].as_str().expect("Missing URL field");
        let url= url_str.split('/').collect::<Vec<_>>();
       
        let response = Client::new().get(url_str).send().await.unwrap();
        let rjosn :String =   response.text().await.unwrap();
        let mut file = File::create(url.last().unwrap()).await.unwrap();
        file.write_all(rjosn.as_bytes()).await.unwrap();

        let d :HashMap<String,serde_json::Value> = serde_json::from_str(&rjosn).unwrap();
        let down_url = d["downloads"]["client"]["url"].as_str().unwrap();

        let response = Client::new().get(down_url).send().await.unwrap();

        File::create("1.21.5.jar").await.unwrap()
            .write_all(&response.bytes().await.unwrap())
            .await
            .unwrap();



    }
}