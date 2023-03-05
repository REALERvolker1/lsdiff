use lscolors::{LsColors, Style};
use nu_ansi_term::Color;
use std::{
    collections::HashSet,
    env::{args, var},
    fs,
    io::Error,
    path::Path,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

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
    } else if first_arg.contains("-u") {
    }

    if !Path::new(&filepath).exists() {
        println!(
            "path literal '{}' does not exist! Falling back to {}",
            filepath, &home
        );
        filepath = home
    }

    let current_files = read_normal_dir(&filepath)?;

    let diff_file_path = Path::new(&diff_file_path_str);

    if !&diff_file_path.exists() {
        let _writing = write_diff_file(&current_files, &diff_file_path);
        println!(
            "Creating new lsdiff cache file ({}). Run 'lsdiff' again to see normal results.",
            &diff_file_path_str
        );
    } else {
        let (files, diff) = read_diff_file(&diff_file_path)?;

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
                let _diff_file_write = write_diff_file(&current_files, &diff_file_path);
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

// Vec<String>
fn read_diff_file(diff_file: &Path) -> Result<(Vec<String>, bool), Error> {
    let diff_file = fs::read_to_string(diff_file)?;
    let mut diff_lines = diff_file.split("\n");

    let file_time_index = diff_lines.position(|s| s == "[TIME]").unwrap_or(1); // This returns an index starting from 1
    let file_time_str = diff_lines.nth(file_time_index).unwrap_or_else(|| {
        println!("Failed to parse diff file line at file_time_str");
        "0.0"
    });
    let file_time: f64 = file_time_str.parse().unwrap_or_else(|_| {
        println!("Failed to parse diff file float at file_time");
        0.0
    });

    let mut files = Vec::new();
    let file_file_index = diff_lines.position(|s| s == "[FILES]").unwrap_or(2); // This returns an index starting from 1
    for file in diff_lines.skip(file_file_index) {
        files.push(String::from(file));
    }

    let systime = SystemTime::now();
    let unix_time = systime.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let unix_time_days = (unix_time / 60.0 / 60.0 / 24.0).floor();

    let mut diff = false;
    if file_time != unix_time_days {
        diff = true
    }
    /*
    Ok(Diff {
        files: files,
        diff: diff,
    })
    */
    Ok((files, diff))
}

fn write_diff_file(current_files: &Vec<String>, diff_file_path: &Path) -> Result<(), Error> {
    let mut contents: Vec<String> = Vec::new();

    contents.push(String::from("[TIME]"));
    let time = SystemTime::now();
    let unix_time = time.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let unix_time_days = (unix_time / 60.0 / 60.0 / 24.0).floor() as i64;
    contents.push(format!("{:?}", &unix_time_days));

    contents.push(String::from("[FILES]"));
    let file_string = current_files.join("\n");
    contents.push(file_string);

    fs::write(diff_file_path, contents.join("\n"))
}

fn read_normal_dir(folder: &str) -> Result<Vec<String>, Error> {
    let files = fs::read_dir(folder)?;

    let mut current_files: Vec<String> = Vec::new();

    for file_entry in files {
        let file = file_entry?.file_name();
        current_files.push(format!("{}", file.to_str().unwrap())); // potential point of failure
    }
    Ok(current_files)
}
