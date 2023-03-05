use std::{
    collections::HashSet,
    env::{args, var},
    io::Error,
    path::Path,
    process,
};

mod filesystem;
mod output;

struct Icon {
    folder: String,
    file: String,
}

fn main() -> Result<(), Error> {

    let arg = parse_args(&filepath, &diff_file_path_str, &folder_icon, &file_icon);

    if !Path::new(&filepath).exists() {
        println!(
            "path literal '{}' does not exist! Falling back to {}",
            filepath, &home
        );
        filepath = home
    }

    let current_files = filesystem::read_normal_dir(&filepath)?;

    let diff_file_path = Path::new(&diff_file_path_str);

    if !&diff_file_path.exists() {
        let _writing = filesystem::write_diff_file(&current_files, &diff_file_path);
        println!(
            "Creating new lsdiff cache file ({}). Run 'lsdiff' again to see normal results.",
            &diff_file_path_str
        );
    } else {
        let (files, diff) = filesystem::read_diff_file(&diff_file_path, &arg)?;

        if files != current_files {
            let current_full: HashSet<_> = current_files.iter().collect();
            let original_full: HashSet<_> = files.iter().collect();

            let current_diff: Vec<_> = current_full.difference(&original_full).cloned().collect();
            let original_diff: Vec<_> = original_full.difference(&current_full).cloned().collect();

            let current_string: Vec<String> = current_diff.iter().map(|s| s.to_string()).collect();
            let original_string: Vec<String> =
                original_diff.iter().map(|s| s.to_string()).collect();
            output::output(&filepath, &icons, current_string, original_string);
            if diff {
                println!("Saving current directory list to diff");
                let _diff_file_write = filesystem::write_diff_file(&current_files, &diff_file_path);
            }
        }
    }

    Ok(())
}

fn parse_args(
    filepath: &str,
    diff_file_path_str: &str,
    folder_icon: &str,
    file_icon: &str,
) -> String {
    let first_arg = args().nth(1).unwrap_or(String::from(""));
    if first_arg.contains("-h") {
        println!(
            "This program will print the ls diff between a directory now, and the directory state from yesterday."
        );
        println!(
            "'$LSDIFF_DIR': directory to diff (default: '$HOME', current: '{}')",
            filepath
        );
        println!("'$LSDIFF_CACHE': cache file to compare against (default: '$XDG_CACHE_HOME/lsdiff.list', current: '{}')", diff_file_path_str);
        println!(
            "'$LSDIFF_ICON_FOLDER', '$LSDIFF_ICON_FILE': folder/file icons (default: ' ', current:'{} {}')", folder_icon, file_icon
        );
        println!("lsdiff -u -- lets you update the cache");
        process::exit(2);
    }
    first_arg
}
