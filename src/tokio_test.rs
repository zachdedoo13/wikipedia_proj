#![allow(dead_code)]

use std::sync::Arc;
use scraper::Html;
use tokio::sync::{Mutex, Semaphore};
use tokio::task;
use crate::{BErr, read_load_or};
use crate::tree::{Page, WikiTree};
use crate::WIKI;

pub fn tokio_test() -> Result<(), Box<dyn std::error::Error>> {
   let rt = tokio::runtime::Runtime::new()?;
   rt.block_on(tokio_test_inner())?;
   Ok(())
}

pub async fn tokio_test_inner() -> Result<(), Box<dyn std::error::Error>> {
   fn to_url(name: &String) -> String { format!("{WIKI}wiki/{name}") };

   // setup
   let data: Vec<String> = read_load_or("src/saved_data/slimmed_list_vec.txt", || panic!());

   let names = Arc::new(data.iter().map(|e| e.clone()).collect::<Vec<String>>());
   let limit = Arc::new(Semaphore::new(1000));
   let end = Arc::new(Mutex::new(WikiTree::default()));

   // distribute code over threads
   let mut handles = vec![];
   for (i, name) in names.iter().enumerate() {
      let semaphore = Arc::clone(&limit);
      let end = Arc::clone(&end);
      let name = name.clone();
      let handle = task::spawn(async move {
         let _permit = semaphore.acquire().await.unwrap();
         {
            let mut end = end.lock().await;
            let page = load_page_tokio(&to_url(&name)).await;
            println!("Processed {i}");
         }
      });
      handles.push(handle);
   }

   let mut count = 0;

   for handle in handles {
      handle.await.unwrap();
      count += 1;
   }
   println!("Finished {count} entry's");

   Ok(())
}

pub async fn load_html_tokio(name: &String) -> Result<String, BErr> {
   let response = reqwest::get(name).await?;
   let html = response.text().await?;
   Ok(html)
}

pub async fn load_page_tokio(name: &String) -> Result<Html, BErr> {
   let html = load_html_tokio(name).await?;
   let passer = Html::parse_document(&html);
   Ok(passer)
}

async fn load_wiki_page_tokio(name: String) -> Result<(String, Html), BErr> {
   let url = "https://en.wikipedia.org/wiki/".to_owned() + &*name;
   let passer = load_page_tokio(&url).await?;
   Ok((name, passer))
}
pub async fn processes_page_tokio<T: Into<String>>(name: T) -> Result<Page, BErr> {
   let loaded = load_wiki_page_tokio(name.into()).await?;
   let page = crate::tree::find_links_in_page(loaded)?;
   Ok(page)
}