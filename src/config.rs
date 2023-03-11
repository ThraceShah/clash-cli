use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Read, path::PathBuf, str};

//定义一个结构体，读取clash的配置文件
#[derive(Debug, Serialize, Deserialize)]
pub struct ClashConfig {
    #[serde(alias = "external-controller")]
    pub external_controller: String,
    pub secret: String,
}

impl ClashConfig {
    //解析clash的配置文件
    pub fn parse_clash_config(path: PathBuf) -> ClashConfig {
        let mut yaml_str = String::new();
        let mut file = std::fs::File::open(path).unwrap();
        file.read_to_string(&mut yaml_str).unwrap();
        let config: ClashConfig =
            serde_yaml::from_str(yaml_str.as_str()).expect("config.yaml read failed!");
        return config;
    }
}

//定义一个结构体，表示JSON数据中的每一项
#[derive(Serialize, Deserialize, Clone)]
pub struct Proxy {
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub history: Vec<History>,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub proxy_type: String,
    #[serde(default)]
    pub udp: bool,
    #[serde(default)]
    pub ave_delay: usize,
}
impl Proxy {
    pub fn get_history_average_delay(self: &mut Proxy) -> usize {
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

    pub fn get_history_mean_delay(self: &mut Proxy) -> usize {
        match self.history.len() {
            0 => match self.proxy_type.as_str() {
                "URLTest" => self.ave_delay = 400,
                "Direct" => self.ave_delay = 0,
                "Reject" => self.ave_delay = 0,
                _ => self.ave_delay = 10000,
            },
            _ => {
                let mut result = 0;
                for item in &self.history {
                    match item.delay {
                        0 => result = result + 10000,
                        _ => result = result + item.delay,
                    }
                }
                self.ave_delay = result / self.history.len();
            }
        }
        return self.ave_delay;
    }
}

//定义一个结构体，表示history中的每一项
#[derive(Serialize, Deserialize, Clone)]
pub struct History {
    pub time: String,
    pub delay: usize,
    #[serde(alias = "meanDelay", default)]
    pub mean_delay: usize,
}

//定义一个结构体，表示proxies中的每一项
#[derive(Serialize, Deserialize, Clone)]
pub struct Proxies {
    pub proxies: HashMap<String, Proxy>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Provider {
    pub name: String,
    #[serde(default)]
    pub proxies: Vec<Proxy>,
    #[serde(alias = "type")]
    pub provider_type: String,
    #[serde(alias = "vehicleType", default)]
    pub vehicle_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Providers {
    pub providers: HashMap<String, Provider>,
}
