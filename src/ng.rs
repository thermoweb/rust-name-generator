use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::PathBuf;

use rand::distributions::WeightedIndex;
use rand::prelude::{Distribution, SliceRandom};
use rand::thread_rng;

const SLICE_WINDOW_SIZE: usize = 3;

/// Generate a random name from precomputed transition map
///
/// # Arguments
///
/// * `transition_map`: precomputed transition map
///
/// returns: String
pub fn generate_name(transition_map: &HashMap<String, HashMap<char, i32>>) -> String {
    let mut generated_name = Vec::new();
    generated_name.push(choose_first_letter(&transition_map));

    while let Some(next) = get_next_letter(&generated_name, &transition_map) {
        generated_name.push(next);
    }
    String::from_iter(generated_name)
}

fn choose_first_letter(map: &HashMap<String, HashMap<char, i32>>) -> char {
    let starting_letters: Vec<_> = map.keys()
        .filter(|k| k.len() == 1)
        .collect();
    let random_letter = starting_letters.choose(&mut thread_rng()).unwrap();
    random_letter.chars().next().unwrap()
}

fn get_next_letter(name: &Vec<char>, map: &HashMap<String, HashMap<char, i32>>) -> Option<char> {
    let (start, end) = if name.len() > SLICE_WINDOW_SIZE { (name.len() - SLICE_WINDOW_SIZE, name.len()) } else { (0, name.len()) };
    let slice = name[start..end].iter().cloned().collect::<String>();
    let next_letters = map.get(&slice).unwrap();
    let next_tuple: Vec<(&char, &i32)> = next_letters.iter().map(|(k, v)| (k, v)).collect();
    let dist = WeightedIndex::new(next_tuple.iter().map(|item| item.1)).unwrap();
    let next = *next_tuple[dist.sample(&mut thread_rng())].0;
    if next == '\0' {
        return None;
    }
    Some(next)
}

/// Load transition map from string.
///
/// # Arguments
///
/// * `map`: string map to load
///
/// returns: Result<HashMap<String, HashMap<char, i32, RandomState>, RandomState>, Error>
pub fn read_map_from_resource(map: &str) -> Result<HashMap<String, HashMap<char, i32>>, Error> {
    Ok(serde_json::from_str(map)?)
}

/// Load transition map from file.
///
/// # Arguments
///
/// * `path`: File path to load.
///
/// returns: Result<HashMap<String, HashMap<char, i32, RandomState>, RandomState>, Error>
pub fn read_map_from_file(path: &PathBuf) -> Result<HashMap<String, HashMap<char, i32>>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let map = serde_json::from_reader(reader)?;
    Ok(map)
}

/// Compute transition map from list of names in a file
///
/// # Arguments
///
/// * `path`: File path to name list to compute
///
/// returns: Result<HashMap<String, HashMap<char, i32, RandomState>, RandomState>, Error>
pub fn compute_transitions_map(path: &PathBuf) -> Result<HashMap<String, HashMap<char, i32>>, Error> {
    let mut transitions = HashMap::new();
    let file = File::open(path)?;
    let file_size = file.metadata().unwrap().len();
    let reader = BufReader::new(file);
    let pb = indicatif::ProgressBar::new(file_size);

    pb.println("[+] reading file");
    for line in reader.lines() {
        let firstname = String::from(line?);
        let line_size = firstname.chars().count() as u64;
        if line_size < SLICE_WINDOW_SIZE as u64 {
            //TODO: handle short names
        } else {
            let text_vec = firstname.chars().collect::<Vec<_>>();
            for (i, _item) in text_vec.iter().enumerate() {
                let start = if i >= SLICE_WINDOW_SIZE { i - SLICE_WINDOW_SIZE + 1 } else { 0 };
                let slice = text_vec[start..i + 1].iter().cloned().collect::<String>();
                let next_letter = if i + 1 < line_size as usize { text_vec.get(i + 1).unwrap().clone() } else { '\0' };
                let next_stats = transitions.entry(slice.clone()).or_insert(HashMap::new());
                let count = next_stats.entry(next_letter).or_insert(0);
                *count += 1;
            }
        }
        pb.inc(firstname.len() as u64);
    }
    pb.finish_and_clear();
    pb.println("done");

    println!("{:?}", transitions);

    Ok(transitions)
}
