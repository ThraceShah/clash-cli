mod config;

use anyhow::Result;
use config::{ClashConfig, Proxies, Proxy};
use hyper::{Body, Client, Method};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use platform_dirs::AppDirs;
use serde_json::json;
use std::{
    io::{self},
    path::{Path, PathBuf},
    process::exit,
    str,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            get_nodes().await.expect("msg");
        }
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
                    let msg = get_api(args[2].as_str()).await.expect("msg");
                    println!("{}", msg);
                }
                _ => {}
            }
        }
        4 => {
            let arg = args[1].as_str();
            match arg {
                "put" => {
                    put_api(args[2].as_str(), args[3].as_str())
                        .await
                        .expect("msg");
                }
                _ => {}
            }
        }
        _ => {
            println!("help");
        }
    }
}

//获取所有可用的代理分组，并和用户交互
async fn get_nodes() -> Result<String> {
    let proxies_str = get_api("proxies").await?;
    let mut proxies: Proxies = serde_json::from_str(proxies_str.as_str()).unwrap();
    let mut auto_names = Vec::new();
    let mut sel_names = Vec::new();
    for pair in &mut proxies.proxies {
        pair.1.get_history_average_delay();
        match pair.1.proxy_type.as_str() {
            "Selector" => {
                sel_names.push(pair.1.name.clone());
            }
            "URLTest" => {
                auto_names.push(pair.1.name.clone());
            }
            _ => {}
        }
    }
    auto_names.sort();
    // let auto_groups = create_groups(&auto_names, &proxies);
    sel_names.sort();
    let sel_groups = create_groups(&sel_names, &proxies);
    print_selectors_info(&sel_groups).await;
    Ok("".to_string())
}

//打印selector组的信息，并和用户交互
async fn print_selectors_info(groups: &Vec<(String, Vec<Proxy>)>) {
    println!("请输入要选中的节点序号:");
    let mut index = 0;
    for pair in groups {
        println!("{}:{}", index, pair.0);
        index = index + 1;
    }
    println!("其他:退出程序!");
    let mut num_str = String::new();
    io::stdin().read_line(&mut num_str).expect("not a num");
    match num_str.trim().parse::<usize>() {
        Ok(i) => {
            if i >= groups.len() {
                exit(0x0100);
            }
            print_proxy_info(groups[i].0.to_string(), &groups[i].1).await;
        }
        Err(..) => {
            exit(0x0100);
        }
    }
}

//打印代理信息，并和用户交互
async fn print_proxy_info(selector: String, proxys: &Vec<Proxy>) {
    println!("请输入要选中的代理序号:");
    let mut index = 0;
    let mut usable = Vec::new();
    for proxy in proxys {
        if proxy.ave_delay > 2000 {
            continue;
        }
        usable.push(proxy.clone());
        println!("{}:{},delay:{}ms", index, proxy.name, proxy.ave_delay);
        index = index + 1;
    }
    println!("其他:退出程序!");
    let mut num_str = String::new();
    io::stdin().read_line(&mut num_str).expect("not a num");
    match num_str.trim().parse::<usize>() {
        Ok(i) => {
            if i >= usable.len() {
                exit(0x0100);
            }
            let api = format!("proxies/{}", selector);
            let result = put_api(api.as_str(), usable[i].name.as_str()).await;
            // let result = put_api("proxies",selector.as_str(), usable[i].name.as_str()).await;
            match result {
                Ok(..) => {}
                Err(msg) => {
                    print!("{}", msg);
                }
            }
        }
        Err(..) => {
            exit(0x0100);
        }
    }
}

//根据selector的名字创建对应的多个组
fn create_groups(group_names: &Vec<String>, proxies: &Proxies) -> Vec<(String, Vec<Proxy>)> {
    let mut result: Vec<(String, Vec<Proxy>)> = Vec::new();
    for group_name in group_names {
        let root = &proxies.proxies.get(group_name).unwrap();
        let group = add_proxy_to_group(&root.all, proxies);
        result.push((group_name.to_string(), group));
    }
    result
}

//把代理节点添加到一个selector组里面
fn add_proxy_to_group(group_sub_names: &Vec<String>, proxies: &Proxies) -> Vec<Proxy> {
    let mut result: Vec<Proxy> = Vec::new();
    for name in group_sub_names {
        for pair in &proxies.proxies {
            if pair.1.name == *name {
                result.push(pair.1.clone());
            }
        }
    }
    result.sort_by(|a, b| b.ave_delay.cmp(&a.ave_delay));
    result
}

//对get请求的封装
async fn get_api(api: &str) -> Result<String> {
    let client = Client::new();
    let config_path = get_config_path();
    let mut config = ClashConfig::parse_clash_config(config_path);
    if config.external_controller.starts_with("0.0.0.0") {
        config.external_controller = config.external_controller.replace("0.0.0.0", "127.0.0.1");
    }
    let request = utf8_percent_encode(api, NON_ALPHANUMERIC).to_string();
    let uri = format!("http://{}/{}", config.external_controller, request);
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
    // println!("{}", content);
    let r = content.to_string();
    Ok(r)
}

//对put 请求的封装
async fn put_api(api: &str, param: &str) -> Result<()> {
    let client = Client::new();
    let config_path = get_config_path();
    let mut config = ClashConfig::parse_clash_config(config_path);
    if config.external_controller.starts_with("0.0.0.0") {
        config.external_controller = config.external_controller.replace("0.0.0.0", "127.0.0.1");
    }
    let parts: Vec<&str> = api.split('/').collect();
    let request = parts
        .iter()
        .map(|s| utf8_percent_encode(s, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<String>>()
        .join("/");
    let uri = format!("http://{}/{}", config.external_controller, request);
    let secret = format!("Bearer {}", config.secret);
    let json_body = json!({ "name": param });
    let json_str = json_body.to_string();
    let req = hyper::Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", secret))
        .body(Body::from(json_str))
        .expect("msg");
    let res = client.request(req).await?;
    if !res.status().is_success() {
        return Err(anyhow::format_err!("{}", res.status()));
    }
    println!("修改成功!");
    Ok(())
}

//获取clash的配置文件，依次查找程序运行目录和~/.config/clash目录
fn get_config_path() -> PathBuf {
    let excute_file = std::env::args().nth(0).expect("msg");
    let excute_path = Path::new(&excute_file);
    let mut config_file = excute_path.parent().unwrap().join("config.yaml");
    match config_file.is_file() {
        false => {
            let user_dir = AppDirs::new(Some("clash"), false).unwrap();
            // let str = user_dir.config_dir.to_str();
            let mut config_dir = user_dir.config_dir.parent().unwrap();
            if config_dir.ends_with("Roaming/") {
                config_dir = config_dir.parent().unwrap().parent().unwrap();
            } else {
                config_dir = config_dir.parent().unwrap();
            }
            config_file = config_dir.join(".config").join("clash").join("config.yaml");
            if !config_file.is_file() {
                println!("config.yaml not found");
                exit(0x0100);
            }
        }
        true => {}
    }
    return config_file;
}
