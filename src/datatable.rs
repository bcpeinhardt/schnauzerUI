//! This module contains code for working with Schnauzer UI datatables.
//! 
//! In Schnauzer UI, a "datatable" is just a csv file with variable values for use in running
//! a script. The first row of the csv, i.e. the headers, should contain the variables names
//! you'd like to use in your script.
//! Each record after that in the csv file represents a test run.
//! 
//! For example, the csv file
//! ```csv
//! username, password
//! test@test.com, pa$$word
//! test2@test,com, 123456
//! ``` 
//! 
//! and the sui file
//! ```sui
//! url "https://mywebsite.com"
//! locate "email" and type "<username>"
//! locate "password" and type "<password>" 
//! ```
//! 
//! when combined will become the script
//! ```sui
//! url "https://mywebsite.com"
//! locate "email" and type "test@test.com"
//! locate "password" and type "pa$$word" 
//! 
//! url "https://mywebsite.com"
//! locate "email" and type "test2@test,com"
//! locate "password" and type "123456" 
//! ```

use anyhow::{bail, Context, Result};
use std::{collections::HashMap, path::Path};

/// Reads in a csv file in the format for a SchnauzerUI datatables.
pub fn read_csv(path: impl AsRef<Path>) -> Result<Vec<HashMap<String, String>>> {
    let mut rdr = csv::Reader::from_path(path).context("Could not find the specified datatable")?;

    let headers = rdr
        .headers()?
        .iter()
        .map(|s| s.trim().to_owned())
        .collect::<Vec<_>>();

    let mut variable_runs = vec![];

    for record in rdr.records() {
        let mut hm: HashMap<String, String> = HashMap::new();
        let mut record = record?;
        record.trim(); // This is more useful than allowing leading and trailing whitespace
        for (j, item) in record.iter().enumerate() {
            let Some(header) = headers.get(j) else {
                bail!("This record is not the same length as the header row. Are you missing a header for this value?")
            };
            let _ = hm.insert(header.to_owned(), item.to_owned());
        }
        variable_runs.push(hm);
    }
    Ok(variable_runs)
}

/// Takes a schanuzerUI script with datatable variables and inlines the variables
/// into the script.
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
