use std::collections::HashSet;
use std::fs;
use std::time::Instant;

use scraper::{Html, Selector};
use serde_json::to_string_pretty;

use crate::{BErr, load_page, read_load_or, SerHash, timer_var, WIKI};

#[derive(serde::Serialize, serde::Deserialize)]
struct Prog {
   pub count_pages: u32,
   pub count_entry: u32,
   pub latest: String,
   pub ser: SerHash,
}
impl Default for Prog {
   fn default() -> Self {
      Self {
         count_pages: 0,
         count_entry: 0,
         latest: "w/index.php?title=Special:AllPages&from=Aa".to_string(),
         ser: Default::default(),
      }
   }
}
pub fn generate_list_of_pages(cutoff: usize) -> Result<(), BErr> {
   /// setup
   let mut prog = read_load_or("src/saved_data/page_list", Prog::default);
   let initial_entry_count = prog.count_entry;

   /// load pages until cutoff
   let elapsed_time = timer_var!({
      for _ in 0..cutoff {
         let page = load_page(&(WIKI.to_owned() + &prog.latest))?;
         let (new_entries, next_page) = linked_page_iter(page)?;
         prog.latest = next_page;
         prog.count_entry += new_entries.len() as u32;
         prog.count_pages += 1;
         prog.ser.set.extend(new_entries);
         println!("count_pages -> {} | count_entry's -> {}", prog.count_pages, prog.count_entry);
      }
   }).0;

   /// display page load time
   let new_entries_count = prog.count_entry - initial_entry_count;
   println!("\n{new_entries_count} entry's in {elapsed_time:?}");

   let save_start_time = Instant::now();
   let serialized_prog = to_string_pretty(&prog)?;
   fs::write("src/saved_data/page_list", serialized_prog)?;

   println!("Saved in {:?}", save_start_time.elapsed());

   let file_size_in_bytes = fs::metadata("src/saved_data/page_list")?.len();
   let file_size_in_mb = (file_size_in_bytes as f64) / (1024.0 * 1024.0);
   println!("File size is {file_size_in_mb}mb\n");

   Ok(())
}

pub fn linked_page_iter(doc: Html) -> Result<(HashSet<String>, String), BErr> {
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