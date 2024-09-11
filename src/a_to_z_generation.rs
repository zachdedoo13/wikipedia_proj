use std::collections::HashSet;
use std::fs;

use scraper::{Html, Selector};
use serde_json::to_string_pretty;

use crate::{BErr, load_page, read_load_or, SerHash, timer, timer_var, WIKI};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Prog {
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



const FILE_PATH: &str = "src/saved_data/page_list.txt";
/// generate a list of wikipedia pages form A -> Z
pub fn generate_list_of_pages(cutoff: usize) -> Result<(), BErr> {
   // setup
   let mut prog = timer!("Load file time", { read_load_or(FILE_PATH, Prog::default) });
   let initial_entry_count = prog.count_entry;
   println!(); // new line for formating

   // load pages until cutoff
   let elapsed_time = timer_var!({
      for _ in 0..cutoff {
         let page = load_page(&(WIKI.to_owned() + &prog.latest))?;
         let (new_entries, next_page) = linked_page_iter(page)?;
         prog.latest = next_page;
         prog.count_entry += new_entries.len() as u32;
         prog.count_pages += 1;
         prog.ser.set.extend(new_entries);
         println!("count_pages -> {} | count_entry's -> {} | Next page {}", prog.count_pages, prog.count_entry, prog.latest);
      }
   }).0;

   // display page load time
   let new_entries_count = prog.count_entry - initial_entry_count;
   let minutes_elapsed = elapsed_time.as_secs_f32() / 60.0;
   let entry_per_min = new_entries_count as f32 / minutes_elapsed;
   println!("\n{new_entries_count} entry's in {elapsed_time:?}, entry's_per_min = {entry_per_min}e/m");

   // save to disk
   timer!("Save Time", {
      let serialized_prog = to_string_pretty(&prog)?;
      fs::write(FILE_PATH, serialized_prog)?;
   });

   // log stats
   let file_size_in_bytes = fs::metadata(FILE_PATH)?.len();
   let file_size_in_mb = (file_size_in_bytes as f64) / (1024.0 * 1024.0);
   println!("File size is {file_size_in_mb}mb");

   println!(); // new line for formating

   Ok(())
}

/// internal function for generate_list_of_pages()
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

   Ok((found, next_page_link))
}