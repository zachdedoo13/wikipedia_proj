use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

use reqwest;
use scraper::{ElementRef, Html, Selector};
use serde_json::{from_str, to_string_pretty};

pub type BErr = Box<dyn std::error::Error>;

fn main() -> Result<(), Box<dyn std::error::Error>> {

   loop {
      generate_list_of_pages(50)?;
   }

   Ok(())
}


fn size_test() -> Result<(), BErr> {
   let st = Instant::now();
   brute_force_page_list(46)?;
   println!("Data grabbing -> {:?}", st.elapsed());

   let data = std::fs::read_to_string("src/saved_data/page_list")?;
   let deserialized: HashSet<String> = serde_json::from_str::<SerHash>(&data)?.set;

   let st = Instant::now();

   let mut ave = 0.0;
   let tot = deserialized.len() as f32;
   for item in deserialized.iter() {
      let size = size_of_val(item.as_bytes());
      ave += size as f32 / tot;
   }

   println!("Average size in bytes => {ave}, In time => {:?}", st.elapsed());

   Ok(())
}

fn load_page(name: &String) -> Result<Html, BErr> {
   let url = name;

   let response = reqwest::blocking::get(url)?;
   let html = response.text()?;
   let passer = Html::parse_document(&html);
   Ok(passer)
}

fn load_wiki_page(name: String) -> Result<(String, Html), BErr> {
   let url = "https://en.wikipedia.org/wiki/".to_owned() + &*name;

   let response = reqwest::blocking::get(url)?;
   let html = response.text()?;
   let passer = Html::parse_document(&html);
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
   pub fn insert(&mut self, data: Page) {
      self.pages.insert(data.0, data.1);
   }
}


fn brute_force_page_list(sample_size: u32) -> Result<(), BErr> {
   let mut found = HashSet::new();

   let mut stats = 0.0;

   let initial = processes_page("Indium(II)_chloride")?;
   found.extend(initial.1);

   for i in 0..sample_size {
      let temp = processes_page(found.iter().nth(i as usize).unwrap())?;

      stats += temp.1.len() as f32 / sample_size as f32;

      found.extend(temp.1);
      println!("Processed {i}");
   }

   let ser_temp = SerHash { set: found };
   let serialized = serde_json::to_string_pretty(&ser_temp)?;

   std::fs::write("src/saved_data/page_list", serialized).unwrap();

   println!("Average amount -> {stats}");

   Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Prog {
   pub count_pages: u32,
   pub count_entry: u32,
   pub latest: String,
   pub ser: SerHash,
}

const BASE: &str = "https://en.wikipedia.org/";
fn generate_list_of_pages(cutoff: usize) -> Result<(), BErr> {
   let mut found = HashSet::<String>::new();


   let mut count_pages: u32 = 0;
   let mut count_entry: u32 = 0;
   let mut latest = match fs::read_to_string("src/saved_data/page_list") {
      Ok(str) => {
         match from_str::<Prog>(str.as_str()) {
            Ok(prog) => { count_entry = prog.count_entry; count_pages = prog.count_pages; prog.latest },
            Err(_) => "w/index.php?title=Special:AllPages&from=Aa".to_string(),
         }
      },
      Err(_) => "w/index.php?title=Special:AllPages&from=Aa".to_string(),
   };

   let st = Instant::now();
   let sv = count_entry;
   for _ in 0..cutoff  {
      let page = load_page(&(BASE.to_owned() + &*latest))?;
      let check = linked_page_iter(page)?;
      latest = check.1;
      count_entry += check.0.len() as u32;
      count_pages += 1;
      found.extend(check.0);
      println!("count_pages -> {count_pages} | count_entry's -> {count_entry}")
   }
   let et = st.elapsed();
   let ev = count_entry - sv;
   println!("\n{ev} entry's in {et:?}");

   let st = Instant::now();

   let prog = Prog {
      count_pages,
      count_entry,
      latest,
      ser: SerHash { set: found },
   };

   let ser_prog = to_string_pretty(&prog)?;
   fs::write("src/saved_data/page_list", ser_prog).unwrap();

   println!("Saved in {:?}", st.elapsed());

   let size_in_bytes = fs::metadata("src/saved_data/page_list")?.len();
   let size_in_mb = (size_in_bytes as f64) / (1024.0 * 1024.0);
   println!("File size is {size_in_mb}mb\n");

   Ok(())
}

fn linked_page_iter(doc: Html) -> Result<(HashSet<String>, String), BErr> {
   let body_selector = Selector::parse(".mw-redirect")?;

   let found: HashSet<String> = doc.select(&body_selector)
       .filter_map(|e| {
          if let Some(href) = e.value().attr("href") {
             if href.starts_with("/wiki") && !(href.contains(":") || href.contains("#")) {
                let stripped = href.strip_prefix("/wiki/").unwrap();
                Some(stripped.to_string())
             } else {
                None
             }
          } else {
             None
          }
       })
       .collect();

   let next_page_selector = Selector::parse(r#"[title="Special:AllPages"]"#)?;

   let next_page_link = doc.select(&next_page_selector)
       .find(|e| e.text().collect::<String>().starts_with("Next")).unwrap()
       .value().attr("href").unwrap().to_string();


   println!("Next page {next_page_link}");
   Ok((found, next_page_link))
}


#[derive(serde::Serialize, serde::Deserialize)]
struct SerHash {
   pub set: HashSet<String>,
}

