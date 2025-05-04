use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: deduper <path1> <path2> ...");
        std::process::exit(1);
    }

    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut total_processed_size: u64 = 0;
    let mut total_duplicate_size: u64 = 0;
    let mut duplicate_groups: usize = 0;

    for path in &args {
        let p = Path::new(path);
        if !p.exists() || !p.is_dir() {
            eprintln!("Skipping invalid directory: {}", path);
            continue;
        }

        // Recursively scan the current directory and its subdirectories
        let mut current_dir = path.clone();
        print!("🌍 Scanning directory: {}... \r", Path::new(&current_dir).display());
        io::stdout().flush().unwrap(); // Ensure the output is immediately printed

        let mut dir_processed_size: u64 = 0;

        for entry in WalkDir::new(&current_dir).follow_links(false).into_iter().filter_map(Result::ok) {
            let file_type = entry.file_type();
            if file_type.is_file() {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let size = metadata.len();
                dir_processed_size += size;

                files_by_size.entry(size).or_default().push(entry.path().to_path_buf());
            }

            // Update the terminal with the current subdirectory being scanned
            let current_path = entry.path().to_string_lossy();
            if entry.file_type().is_dir() {
                print!("🌍 Currently scanning: {}... \r", current_path);
                io::stdout().flush().unwrap(); // Flush the output immediately to replace the previous line
            }
        }

        total_processed_size += dir_processed_size;
        // After finishing the current directory, summarize progress
        print!(
            "🌍 Finished directory: {} - Total processed: {} bytes - Duplicates found: {} bytes\r",
            Path::new(&current_dir).display(),
            total_processed_size,
            total_duplicate_size
        );
        io::stdout().flush().unwrap();
    }

    // After the scanning loop, we'll summarize the duplicates
    println!("\n🔍 Duplicate files summary:");

    for (size, paths) in &files_by_size {
        if paths.len() > 1 {
            let duplicate_size = size * (paths.len() as u64 - 1);
            total_duplicate_size += duplicate_size;
            duplicate_groups += 1;

            // Only print the size of the group (instead of each file)
            println!("Size: {} bytes → {} duplicate files", size, paths.len());
        }
    }

    println!("\n✅ Finished scanning.");
    println!(
        "Total processed size: {} bytes\nTotal duplicate size: {} bytes\nTotal duplicate groups: {}",
        total_processed_size, total_duplicate_size, duplicate_groups
    );
}
