use anyhow::Result;
use hyper::{Body, Client, Method, Uri};
use std::{ptr::null, str, string, path::Path};
use platform_dirs::{AppDirs};

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
    let config_path=get_config_path();
    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri("http://localhost:60881/")
        .header(
            "Authorization",
            "Bearer 9c59d9fd-1506-4643-96d2-3180adfd6b2a",
        )
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

fn get_config_path()->String{
    let excute_file=std::env::args().nth(0).expect("msg");
    let excute_path=Path::new(&excute_file);
    let mut config_file=excute_path.parent().unwrap().join("config.yaml");
    if(config_file.is_file()==false)
    {
        let file1=config_file.as_path();
        let user_dir = AppDirs::new(Some("clash"),false).unwrap();
        let config_dir=user_dir.config_dir.as_path();
        config_file=config_dir.join("config.yaml");
        let file1=config_file.as_path();
        if !config_file.is_file()
        {
            return "".to_string();
        }
    }
    return "".to_string();
}