use std::collections::{HashMap, HashSet};
use std::mem;
use reqwest;
use scraper::{Html, Selector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let mut tree = WikiTree::default();


   tree.insert(processes_page("Indium(II)_chloride")?);

   println!("{tree:?}");


   Ok(())
}

fn load_wiki_page(name: String) -> Result<(String, Html), Box<dyn std::error::Error>> {
   let url = "https://en.wikipedia.org/wiki/".to_owned() + &*name;

   let response = reqwest::blocking::get(url)?;
   let html = response.text()?;
   let passer = Html::parse_document(&html);
   Ok((name, passer))
}
fn find_links_in_page(loaded: (String, Html)) -> Result<Page, Box<dyn std::error::Error>> {
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
fn processes_page<T: Into<String>>(name: T) -> Result<Page, Box<dyn std::error::Error>> {
   let loaded = load_wiki_page(name.into())?;
   let page = find_links_in_page(loaded)?;
   Ok(page)
}

#[derive(Debug)]
struct Page(String, HashSet<String>);

#[derive(Default, Debug)]
struct WikiTree {
   pub pages: HashMap<String, HashSet<String>>
}
impl WikiTree {
   pub fn insert(&mut self, data: Page) {
      self.pages.insert(data.0, data.1);
   }
}

