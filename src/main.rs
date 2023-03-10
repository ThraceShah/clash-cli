use anyhow::Result;
use hyper::{Body, Client, Method};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    io::{self, Read},
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
                    put_api(args[2].as_str(), args[3].as_str(),"")
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
            // let api = format!("proxies/{}", selector);
            // let result = put_api(api.as_str(), usable[i].name.as_str()).await;
            let result = put_api("proxies",selector.as_str(), usable[i].name.as_str()).await;
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

fn create_groups(group_names: &Vec<String>, proxies: &Proxies) -> Vec<(String, Vec<Proxy>)> {
    let mut result: Vec<(String, Vec<Proxy>)> = Vec::new();
    for group_name in group_names {
        let root = &proxies.proxies.get(group_name).unwrap();
        let group = add_proxy_to_group(&root.all, proxies);
        result.push((group_name.to_string(), group));
    }
    result
}

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

async fn get_api(api: &str) -> Result<String> {
    let client = Client::new();
    let config_path = get_config_path();
    let mut config = parse_clash_config(config_path);
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

// async fn put_api(api: &str, param: &str) -> Result<()> {
//     let client = Client::new();
//     let config_path = get_config_path();
//     let mut config = parse_clash_config(config_path);
//     if config.external_controller.starts_with("0.0.0.0") {
//         config.external_controller = config.external_controller.replace("0.0.0.0", "127.0.0.1");
//     }
//     let request = utf8_percent_encode(api, NON_ALPHANUMERIC).to_string();
//     let uri = format!("http://{}/{}", config.external_controller, request);
//     let secret = format!("Bearer {}", config.secret);
//     let json_body = json!({ "name": param });
//     let json_str = json_body.to_string();
//     let req = hyper::Request::builder()
//         .method(Method::PUT)
//         .uri(uri)
//         .header("Authorization", format!("Bearer {}", secret))
//         .body(Body::from(json_str))
//         .expect("msg");
//     let res = client.request(req).await?;
//     if !res.status().is_success() {
//         return Err(anyhow::format_err!("{}", res.status()));
//     }
//     println!("修改成功!");
//     Ok(())
// }

async fn put_api(api: &str,request:&str, param: &str) -> Result<()> {
    let client = Client::new();
    let config_path = get_config_path();
    let mut config = parse_clash_config(config_path);
    if config.external_controller.starts_with("0.0.0.0") {
        config.external_controller = config.external_controller.replace("0.0.0.0", "127.0.0.1");
    }
    let new_request = utf8_percent_encode(request, NON_ALPHANUMERIC).to_string();
    let uri = format!("http://{}/{}/{}", config.external_controller,api, new_request);
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

//定义一个结构体，表示JSON数据中的每一项
#[derive(Serialize, Deserialize, Clone)]
struct Proxy {
    #[serde(default)]
    all: Vec<String>,
    #[serde(default)]
    history: Vec<History>,
    #[serde(default)]
    name: String,
    #[serde(rename = "type", default)]
    proxy_type: String,
    #[serde(default)]
    udp: bool,
    #[serde(default)]
    ave_delay: usize,
}
impl Proxy {
    fn get_history_average_delay(self: &mut Proxy) -> usize {
        let mut result = 0;
        if self.history.len() == 0 {
            if self.proxy_type == "URLTest" {
                self.ave_delay = 400;
                return 400;
            }
            self.ave_delay = 10000;
            return 10000;
        }
        for item in &self.history {
            if item.delay == 0 {
                result = result + 10000;
            } else {
                result = result + item.delay;
            }
        }
        self.ave_delay = result / self.history.len();
        self.ave_delay
    }
}

//定义一个结构体，表示history中的每一项
#[derive(Serialize, Deserialize, Clone)]
struct History {
    time: String,
    delay: usize,
}

//定义一个结构体，表示proxies中的每一项
#[derive(Serialize, Deserialize, Clone)]
struct Proxies {
    proxies: HashMap<String, Proxy>,
}
