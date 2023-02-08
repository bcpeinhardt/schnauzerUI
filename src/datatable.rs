use std::{collections::HashMap, path::PathBuf};
use anyhow::{Result, bail, Context};

pub fn read_csv(path: PathBuf) -> Result<Vec<HashMap<String, String>>>{
    let mut rdr = csv::Reader::from_path(path).with_context(|| "Could not find the specified datatable")?;
    let headers = rdr
        .headers()?
        .iter()
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();
    let mut variable_runs = vec![];
    for (i, record) in rdr.records().enumerate() {
        let mut hm: HashMap<String, String> = HashMap::new();
        let mut record = record?;
        record.trim(); // This is more useful than allowing leading and trailing whitespace
        for (j, item) in record.iter().enumerate() {
            let header = match headers.get(j) {
                Some(h) => h,
                _ => bail!("This won't happen")
            };
            hm.insert(
                header.to_owned(),
                item.to_owned(),
            );
        }
        variable_runs.push(hm);
    }
    Ok(variable_runs)
}

pub fn preprocess(code: String, dt: Vec<HashMap<String, String>>) -> String {
    let mut new_code = String::new();
    for (i, hm) in dt.into_iter().enumerate() {
        let mut section = code.clone();
        for (key, value) in hm {
            section = section.replace(&format!("<{}>", key), &value);
        }
        new_code.push_str("\n\n");
        new_code.push_str(&format!("# Test Run {}", i));
        new_code.push_str("\n\n");
        new_code.push_str(&section);
        new_code.push_str("\n\n");
    }
    new_code
}
