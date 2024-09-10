use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use reqwest;
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};

use crate::a_to_z_generation::generate_list_of_pages;

mod a_to_z_generation;
mod tree;
mod macros;

pub type BErr = Box<dyn std::error::Error>;

const WIKI: &str = "https://en.wikipedia.org/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
   loop {
      generate_list_of_pages(256)?;
   }
}


/// loads any html website
fn load_page(name: &String) -> Result<Html, BErr> {
   let response = reqwest::blocking::get(name)?;
   let html = response.text()?;
   let passer = Html::parse_document(&html);
   Ok(passer)
}


pub fn read_load_or<S: AsRef<Path>, T: for<'a> Deserialize<'a>, F>(path: S, or_else: F) -> T
where
    F: FnOnce() -> T,
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

pub fn save<T: Serialize>(path: &Path, data: &T) -> Result<(), std::io::Error> {
   let str = to_string_pretty(data).unwrap();
   fs::write(path, str)?;

   Ok(())
}


/// serializable hashset
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SerHash {
   pub set: HashSet<String>,
}

