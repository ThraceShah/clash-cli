use anyhow::Result;
use hyper::{Body, Client, Method, Uri};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::exit,
    ptr::{null, null_mut},
    str,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => {
            let arg = args[1].as_str();
            match arg {
                "profiles" => {
                    get_usable_prof().await.expect("msg");
                }
                _ => {}
            }
        }
        _ => {
            println!("help");
        }
    }
}

async fn get_usable_prof() -> Result<()> {
    let client = Client::new();
    let config_path = get_config_path();
    let config = get_clash_config(config_path);
    let uri = format!("http://{}", config.external_controller);
    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", config.secret)
        .body(Body::default())
        .expect("msg");
    let mut res = client.request(req).await?;
    if !res.status().is_success() {
        return Err(anyhow::format_err!("{}", res.status()));
    }
    let body = res.body_mut();
    let buf = hyper::body::to_bytes(body).await?;
    let content = str::from_utf8(buf.as_ref())?;
    println!("{}", content);
    Ok(())
}

fn get_config_path() -> PathBuf {
    let excute_file = std::env::args().nth(0).expect("msg");
    let excute_path = Path::new(&excute_file);
    let mut config_file = excute_path.parent().unwrap().join("config.yaml");
    match config_file.is_file() {
        false => {
            let user_dir = AppDirs::new(Some("clash"), false).unwrap();
            let config_dir = user_dir.config_dir.as_path();
            config_file = config_dir.join("config.yaml");
            if !config_file.is_file() {
                println!("config.yaml not found");
                exit(0x0100);
            }
        }
        true => todo!(),
    }
    return config_file;
}

/// 定义 User 类型
#[derive(Debug, Serialize, Deserialize)]
struct ClashConfig {
    #[serde(alias = "external-controller")]
    external_controller: String,
    secret: String,
}

fn get_clash_config(path: PathBuf) -> ClashConfig {
    let mut yaml_str = String::new();
    let mut file = std::fs::File::open(path).unwrap();
    file.read_to_string(&mut yaml_str).unwrap();
    let config: ClashConfig =
        serde_yaml::from_str(yaml_str.as_str()).expect("app.yaml read failed!");
    return config;
}
