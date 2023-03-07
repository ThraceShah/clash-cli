use anyhow::Result;
use hyper::{Body, Client, Method};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::exit,
    str,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => {
            let arg = args[1].as_str();
            match arg {
                "proxies" => {
                    get_nodes().await.expect("msg");
                }
                _ => {}
            }
        }
        3 => {
            let arg = args[1].as_str();
            match arg {
                "get" => {
                    get_api(args[2].as_str()).await.expect("msg");
                }
                _ => {}
            }
        }
        4 => {
            let arg = args[1].as_str();
            match arg {
                "put" => {
                    put_api(args[2].as_str(),args[3].as_str()).await.expect("msg");
                }
                _ => {}
            }
        }
        _ => {
            println!("help");
        }
    }
}

async fn get_nodes()-> Result<String>{
    let proxies_str=get_api("proxies").await?;
    let proxies=read_json(proxies_str.as_str());
    for pair in proxies.proxies {
        let key=pair.0;
        let v=pair.1;
    }
    Ok("".to_string())
}

async fn get_api(api: &str) -> Result<String> {
    let client = Client::new();
    let config_path = get_config_path();
    let config = parse_clash_config(config_path);
    let uri = format!("http://{}/{}", config.external_controller, api);
    let secret = format!("Bearer {}", config.secret);
    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", secret)
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
    let r=content.to_string();
    Ok(r)
}

async fn put_api(api: &str,param:&str) -> Result<()> {
    let client = Client::new();
    let config_path = get_config_path();
    let config = parse_clash_config(config_path);
    let uri = format!("http://{}/{}", config.external_controller, api);
    let secret = format!("Bearer {}", config.secret);
    let req = hyper::Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", secret))
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

#[derive(Debug, Serialize, Deserialize)]
struct ClashConfig {
    #[serde(alias = "external-controller")]
    external_controller: String,
    secret: String,
}

fn parse_clash_config(path: PathBuf) -> ClashConfig {
    let mut yaml_str = String::new();
    let mut file = std::fs::File::open(path).unwrap();
    file.read_to_string(&mut yaml_str).unwrap();
    let config: ClashConfig =
        serde_yaml::from_str(yaml_str.as_str()).expect("config.yaml read failed!");
    return config;
}

fn read_json(raw_json:&str) -> Proxies {
    let parsed: Proxies = serde_json::from_str(raw_json).unwrap();
    return parsed
}

//定义一个结构体，表示JSON数据中的每一项
#[derive(Serialize, Deserialize)]
struct Proxy {
    history: Vec<History>,
    name: String,
    #[serde(rename = "type")]
    proxy_type: String,
    udp: bool,
}

//定义一个结构体，表示history中的每一项
#[derive(Serialize, Deserialize)]
struct History {
    time: String,
    delay: i32,
}

//定义一个结构体，表示proxies中的每一项
#[derive(Serialize, Deserialize)]
struct Proxies {
    proxies: std::collections::HashMap<String, Proxy>,
}