#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use scraper::{Html, Selector};
use crate::{BErr, load_page};

fn load_wiki_page(name: String) -> Result<(String, Html), BErr> {
   let url = "https://en.wikipedia.org/wiki/".to_owned() + &*name;
   let passer = load_page(&url)?;
   Ok((name, passer))
}

fn find_links_in_page(loaded: (String, Html)) -> Result<Page, BErr> {
   let contents_selector = Selector::parse("#mw-content-text")?;

   let doc = loaded.1;

   let body = doc
       .select(&contents_selector)
       .next()
       .ok_or("Country info box element not found!")?;

   let link_a_selector = Selector::parse("a[href][title]")?;

   let mut found = HashSet::new();
   for element in body.select(&link_a_selector) {
      if let Some(href) = element.value().attr("href") {
         if href.starts_with("/wiki") && !(href.contains(":") || href.contains("#")) {
            let stripped = href.strip_prefix("/wiki/").unwrap();
            found.insert(stripped.to_string());
         }
      }
   };

   Ok(Page(loaded.0, found))
}

fn processes_page<T: Into<String>>(name: T) -> Result<Page, BErr> {
   let loaded = load_wiki_page(name.into())?;
   let page = find_links_in_page(loaded)?;
   Ok(page)
}

#[derive(Debug)]
struct Page(String, HashSet<String>);

#[derive(Default, Debug)]
struct WikiTree {
   pub pages: HashMap<String, HashSet<String>>,
}
impl WikiTree {
   // pub fn load() -> Self {
   //
   // }
   pub fn insert(&mut self, data: Page) {
      self.pages.insert(data.0, data.1);
   }
}