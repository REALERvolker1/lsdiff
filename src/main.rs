use lscolors::{LsColors, Style};
use nu_ansi_term::Color;
use std::{
    collections::HashSet,
    env::{args, var},
    io::Error,
    path::Path,
    process,
};

mod filesystem;

struct Icon {
    folder: String,
    file: String,
}

fn main() -> Result<(), Error> {
    let first_arg = args().nth(1).unwrap_or(String::from(""));

    let home = var("HOME").unwrap();
    let cache_home = var("XDG_CACHE_HOME").unwrap_or(format!("{}/.cache", &home));
    let mut filepath = var("LSDIFF_DIR").unwrap_or(home.clone());
    let diff_file_path_str = var("LSDIFF_CACHE").unwrap_or(format!("{}/lsdiff.list", cache_home));
    let icons = Icon {
        folder: var("LSDIFF_ICON_FOLDER").unwrap_or(String::from("")),
        file: var("LSDIFF_ICON_FILE").unwrap_or(String::from("")),
    };

    if first_arg.contains("-h") {
        println!(
            "This program will print the ls diff between a directory now, and the directory state from yesterday."
        );
        println!(
            "'$LSDIFF_DIR': directory to diff (default: '$HOME', current: '{}')",
            &filepath
        );
        println!("'$LSDIFF_CACHE': cache file to compare against (default: '$XDG_CACHE_HOME/lsdiff.list', current: '{}')", &diff_file_path_str);
        println!(
            "'$LSDIFF_ICON_FOLDER', '$LSDIFF_ICON_FILE': folder/file icons (default: ' ', current:'{} {}')", &icons.folder, &icons.file
        );
        println!("lsdiff -u -- lets you update the cache");
        process::exit(2);
    }

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
        let (files, diff) = filesystem::read_diff_file(&diff_file_path, &first_arg)?;

        if files != current_files {
            let current_full: HashSet<_> = current_files.iter().collect();
            let original_full: HashSet<_> = files.iter().collect();

            let current_diff: Vec<_> = current_full.difference(&original_full).cloned().collect();
            let original_diff: Vec<_> = original_full.difference(&current_full).cloned().collect();

            let current_string: Vec<String> = current_diff.iter().map(|s| s.to_string()).collect();
            let original_string: Vec<String> =
                original_diff.iter().map(|s| s.to_string()).collect();
            output(&filepath, &icons, current_string, original_string);
            if diff {
                println!("Saving current directory list to diff");
                let _diff_file_write = filesystem::write_diff_file(&current_files, &diff_file_path);
            }
        }
    }

    Ok(())
}

fn output(basepath: &str, icons: &Icon, current: Vec<String>, original: Vec<String>) -> () {
    let lscolors = LsColors::from_env().unwrap_or_default();

    let output_format = |file: String, op: String| {
        let path_str = format!("{}/{}", &basepath, &file);
        let path = Path::new(&path_str);
        let style = lscolors.style_for_path(&path_str);

        let ansi_style = style.map(Style::to_nu_ansi_term_style).unwrap_or_default();

        let mut output_str = file;
        if path.is_dir() {
            output_str = format!("{} {}", &icons.folder, output_str)
        } else {
            output_str = format!("{} {}", &icons.file, output_str)
        }

        format!("{} {}", op, ansi_style.paint(&output_str))
    };

    let mut current_output: Vec<String> = Vec::new();
    let mut original_output: Vec<String> = Vec::new();

    for file in current {
        current_output.push(output_format(
            file,
            Color::LightGreen.paint("+").to_string(),
        ));
    }
    for file in original {
        original_output.push(output_format(file, Color::Red.paint("-").to_string()));
    }
    let mut i = 0;
    let nustr = Color::DarkGray.paint("---").to_string();
    while i < current_output.len() || i < original_output.len() {
        let cur = current_output.get(i).unwrap_or(&nustr);
        let ori = original_output.get(i).unwrap_or(&nustr);
        println!("{:<20}\t{:>20}", cur, ori);
        i += 1;
    }
}
