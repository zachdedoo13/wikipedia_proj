use std::collections::HashSet;
use std::fs;
use std::future::Future;
use std::path::Path;

use reqwest;
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str, to_string_pretty};
use tokio::task;
use tokio::task::JoinHandle;

use crate::a_to_z_generation::Prog;

mod a_to_z_generation;
mod tree;
mod macros;

pub type BErr = Box<dyn std::error::Error>;

const WIKI: &str = "https://en.wikipedia.org/";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   // loop {
   //    generate_list_of_pages(4)?;
   // }

   let data: Vec<String> = read_load_or("src/saved_data/slimmed_list_vec.txt", || panic!());
   let urls: Vec<String> = data.iter().map(|e| format!("{WIKI}wiki/{e}")).collect();

   let handles: Vec<_> = urls.into_iter().map(|url| {
      // Spawn an async task for each request
      let handle = task::spawn(async move {
         match load_html_tokio(&url).await {
            Ok(html) => Some(html),
            Err(e) => None,
         }
      });
      handle
   }).collect();

   let mut count = 0;

   for handle in handles {
      match handle.await {
         Ok(result) => match result {
            Some(content) => { println!("Worked"); count += 1; }, // Print first 100 chars of content
            None => eprintln!("Error"),
         },
         Err(e) => eprintln!("Task join error: {:?}", e),
      }
   }

   println!("{count}");


   Ok(())
}

/// tokio versions of functions
async fn load_html_tokio(name: &String) -> Result<String, BErr> {
   let response = reqwest::get(name).await?;
   let html = response.text().await?;
   Ok(html)
}




/// loads any html website
fn load_html(name: &String) -> Result<String, BErr> {
   let response = reqwest::blocking::get(name)?;
   let html = response.text()?;
   Ok(html)
}

fn load_page(name: &String) -> Result<Html, BErr> {
   let html = load_html(name)?;
   let passer = Html::parse_document(&html);
   Ok(passer)
}


pub fn read_load_or<T, S>(path: S, or_else: fn() -> T) -> T
where
    T: DeserializeOwned,
    S: AsRef<Path>,
{
   match fs::read_to_string(path.as_ref()) {
      Ok(ser) => {
         let temp = from_str::<T>(ser.as_str());
         match temp {
            Ok(t) => t,
            Err(_) => or_else(),
         }
      }
      Err(_) => or_else(),
   }
}

pub fn save<T, S>(path: S, data: &T) -> Result<(), std::io::Error>
where
    T: Serialize,
    S: AsRef<Path>,
{
   let str = to_string_pretty(data).unwrap();
   fs::write(path.as_ref(), str)?;

   Ok(())
}


/// serializable hashset
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SerHash {
   pub set: HashSet<String>,
}

