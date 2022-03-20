extern crate log;
extern crate yaml_rust;

use log::info;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use yaml_rust::yaml;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlLoader;

struct WebsiteConfig {
    // URL to make requests to
    website_url: String,

    // Name of the website, used in webhooks
    website_name: String,

    // Time between requests in millisecons
    update_interval: i64,

    // String in HTML that will only appear when product is out of stock
    out_of_stock_text: String,
}

impl fmt::Display for WebsiteConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "URL: {},\nName: {},\nUpdate interval (ms): {},\nOut of stock text: {}",
            self.website_url, self.website_name, self.update_interval, self.out_of_stock_text
        )
    }
}

impl WebsiteConfig {
    fn is_in_stock(&self) -> bool {
        let code: String = get_website((self.website_url).to_string());
        !code.contains(&self.out_of_stock_text)
    }
}

fn get_website(url: String) -> String {
    println!("Making request to {}", url);
    let res = reqwest::blocking::get(url.to_string());
    match res {
        Ok(e) => return e.text().unwrap(),
        Err(e) => println!("Error making request to {}: {}", url, e),
    }
    println!("Error!");
    return "".to_string();
}

fn load_yaml_file(file: &str) -> Yaml {
    let mut file = File::open(file).expect("Unable to open config.yaml");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    return YamlLoader::load_from_str(&contents).unwrap()[0].clone();
}

fn main() {
    env_logger::init();

    let file = "config.yaml";
    let config = load_yaml_file(file);
    let websites = config["Websites"].clone();
    let mut website_configs: Vec<WebsiteConfig> = vec![];

    match websites {
        yaml::Yaml::Array(ref v) => {
            for x in v {
                website_configs.push(WebsiteConfig {
                    website_url: x["URL"].as_str().unwrap().to_string(),
                    website_name: x["name"].as_str().unwrap().to_string(),
                    update_interval: x["interval"].as_i64().unwrap(),
                    out_of_stock_text: x["no_stock_indicator"].as_str().unwrap().to_string(),
                })
            }
        }
        _ => {
            println!("lol {:?}", websites);
        }
    };

    for site in website_configs {
        info!("Starting monitor for {}", site.website_name);
        thread::spawn(move || loop {
            if site.is_in_stock() {
                println!("Is in stock on {}!", site.website_url)
            }
            thread::sleep(Duration::from_millis(
                site.update_interval.try_into().unwrap(),
            ));
        });
    }
    loop {}
}
