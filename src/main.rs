use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    process,
};

fn invalid_usage() -> ! {
    eprintln!("usage: {} <path> [--delete]", env::args().next().unwrap());
    process::exit(1);
}

struct Args {
    path: String,
    delete: bool,
}

fn parse_args() -> Args {
    let mut path = None;
    let mut delete = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--delete" | "-d" => delete = true,
            _ => match path {
                None => path = Some(arg),
                Some(_) => invalid_usage(),
            },
        }
    }

    let path = path.unwrap_or_else(|| invalid_usage());

    Args { path, delete }
}

struct StoredImage {
    pub path: String,
    pub size: u64,
}

fn main() {
    let args = parse_args();

    let hasher = image_hasher::HasherConfig::new().to_hasher();
    let mut hashes: BTreeMap<Vec<u8>, StoredImage> = BTreeMap::new();

    let mut file_paths = Vec::new();
    for entry in fs::read_dir(args.path).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            continue;
        }
        let path = entry.path();
        file_paths.push(path);
    }
    let file_count = file_paths.len();

    let mut dup_count = 0;
    let mut sim_count = 0;
    let mut uniq_count = 0;

    for (i, path) in file_paths.into_iter().enumerate() {
        let Ok(image) = image::open(&path) else {
            continue;
        };
        let hash = hasher.hash_image(&image);
        let hash = hash.as_bytes().to_vec();

        let path_string = path.to_string_lossy().to_string();

        let dup_of = hashes.get(&hash);
        let mut sim_to = None;

        for (other_hash, other_path) in &hashes {
            let mut diff_bits = 0;
            for (a, b) in hash.iter().zip(other_hash.iter()) {
                diff_bits += (a ^ b).count_ones();
            }
            if diff_bits <= 5 {
                sim_to = Some(other_path);
                break;
            }
        }

        let mut previous_stored_image = None;
        let display = if let Some(dup_of) = dup_of {
            let dup_of_path = &dup_of.path;
            previous_stored_image = Some(dup_of);
            format!("\x1b[91mdup\x1b[m {path_string} == {dup_of_path}")
        } else if let Some(sim_to) = sim_to {
            let sim_to_path = &sim_to.path;
            previous_stored_image = Some(sim_to);
            format!("\x1b[93msim\x1b[m {path_string} ~= {sim_to_path}")
        } else {
            "".to_string()
        };

        let hash_hex = hex::encode(&hash);

        print!(
            "{}/{file_count} \x1b[90m{hash_hex}\x1b[m {display}\r",
            i + 1
        );
        io::stdout().flush().unwrap();
        if dup_of.is_some() || sim_to.is_some() {
            println!();
        }

        if dup_of.is_some() {
            dup_count += 1;
        } else if sim_to.is_some() {
            sim_count += 1;
        } else {
            uniq_count += 1;
        }

        let file_size = fs::metadata(&path).unwrap().len();

        let mut should_insert = true;

        if args.delete {
            if let Some(previous_stored_image) = previous_stored_image {
                // delete the smaller file
                let path_to_delete = if file_size > previous_stored_image.size {
                    &previous_stored_image.path
                } else {
                    should_insert = false;
                    &path_string
                };
                fs::remove_file(path_to_delete).unwrap();
            }
        }

        if should_insert {
            hashes.insert(
                hash,
                StoredImage {
                    path: path_string,
                    size: file_size,
                },
            );
        }
    }

    // extra spaces at the end to remove any possible leftover characters :)
    println!(
        "{dup_count} \x1b[91mdup\x1b[m, {sim_count} \x1b[93msim\x1b[m, {uniq_count} \x1b[96muniq\x1b[m        ",
    );
}
