#![allow(dead_code)]

use std::collections::{HashSet};
use std::fs;
use std::future::Future;
use std::path::Path;
use reqwest;
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str, to_string_pretty};
use crate::tokio_test::tokio_test;

mod a_to_z_generation;
mod tree;
mod macros;
mod tokio_test;

pub type BErr = Box<dyn std::error::Error>;

pub const WIKI: &str = "https://en.wikipedia.org/";


fn main() -> Result<(), Box<dyn std::error::Error>> {
   tokio_test()?;

   Ok(())
}


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

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SerHash {
   pub set: HashSet<String>,
}