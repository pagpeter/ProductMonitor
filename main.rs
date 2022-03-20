extern crate yaml_rust;

use chrono::{Utc};
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
        if code == "" {
            return false;
        }
        !code.contains(&self.out_of_stock_text)
    }

    fn clone(&self) -> WebsiteConfig {
        WebsiteConfig {
            website_url: self.website_url.clone(),
            website_name: self.website_name.clone(),
            update_interval: self.update_interval,
            out_of_stock_text: self.out_of_stock_text.clone(),
        }
    }
}

fn get_website(url: String) -> String {
    println!("[{}]  Making request to {}", Utc::now(), url);
    let res = reqwest::blocking::get(url.to_string());
    match res {
        Ok(e) => return e.text().unwrap(),
        Err(e) => println!("Error making request to {}: {}", url, e),
    }
    println!("Error!");
    return "".to_string();
}

fn send_webhook(url: String, site: WebsiteConfig) {
    println!("[{}]  Sending webhook", Utc::now());
    let client = reqwest::blocking::Client::new();

    let msg: String = "{\"username\": \"🖥  - Monitor\", \"embeds\": [ {\"title\": \"Monitor triggered\",\"color\": 1841963, \"description\": \"The product is available on ".to_owned() + &site.website_name.to_owned() + "\",\"url\": \"" + &site.website_url.to_owned() + "\",\"footer\": {\"text\": \"built by peet with ❤️\"}}]}";

    println!("{}" , msg);

    let res = client.post(url)
        .header("Content-Type", "application/json")
        .body(msg)
        .send();
    println!("[{}]  Response: {}", Utc::now(), res.unwrap().text().unwrap());
}


fn load_yaml_file(file: &str) -> Yaml {
    let mut file = File::open(file).expect("Unable to open config.yaml");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    return YamlLoader::load_from_str(&contents).unwrap()[0].clone();
}

fn main() {
    let file = "config.yaml";
    let config = load_yaml_file(file);
    let websites = config["websites"].clone();
    let mut website_configs: Vec<WebsiteConfig> = vec![];

    let webhook_url = config["webhook"].as_str().unwrap().to_string();

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
            println!("Error: {:?}", websites);
        }
    };

    for site in website_configs {
        println!("✨ Starting monitor for {}", site.website_name);
        let url = webhook_url.clone();
        // let update_interval = site.clone().update_interval.try_into().unwrap();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            loop {
                // let clone = site.clone();
                if site.is_in_stock() {
                    // println!("🚀 Is in stock on {}!", site.website_url);
                    send_webhook(url.to_string(), site.clone());
                }
                thread::sleep(Duration::from_millis(
                    site.update_interval.try_into().unwrap(),
                ));
            }
        });
    }
    loop {}
}
