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
//mod filesystem;
//mod output;

pub struct State {
    path: String,
    path_list: Vec<String>,
    cache: String,
    cache_list: Vec<String>,
    icon_folder: String,
    icon_file: String,
    diff: bool,
    time: f64,
}

impl State {
    fn get_path_state(&mut self) -> Result<(), Error> {
        let files = fs::read_dir(&self.path)?;
        for file_entry in files {
            let file = file_entry?.file_name();
            self.path_list.push(format!("{}", file.to_str().unwrap())); // potential point of failure
        }
        Ok(())
    }
    fn get_diff_state(&mut self) -> Result<(), Error> {
        let diff_file = fs::read_to_string(&self.cache)?;
        let mut diff_lines = diff_file.split("\n");
        let file_time_index = diff_lines.position(|s| s == "[TIME]").unwrap_or(1);
        let file_time = diff_lines
            .nth(file_time_index)
            .unwrap_or_else(|| {
                println!("Failed to parse diff file line at file_time_str");
                "0.0"
            })
            .parse()
            .unwrap_or_else(|_| {
                println!("Failed to parse diff file float at file_time");
                0.0
            });
        let file_file_index = diff_lines.position(|s| s == "[FILES]").unwrap_or(2);
        for file in diff_lines.skip(file_file_index) {
            self.cache_list.push(String::from(file));
        }
        let current_time = get_time();
        if file_time != current_time {
            self.diff = true;
        }
        Ok(())
    }
    fn write_diff_cache(&self) -> Result<(), Error> {
        fs::write(
            &self.cache,
            format!(
                "[TIME]\n{}\n[FILES]\n{}",
                self.time,
                self.path_list.join("\n")
            ),
        )
    }
    fn compare_lists(&self) -> (Vec<String>, Vec<String>) {
        let current_full: HashSet<_> = self.path_list.iter().collect();
        let original_full: HashSet<_> = self.cache_list.iter().collect();

        let current_diff: Vec<_> = current_full.difference(&original_full).cloned().collect();
        let original_diff: Vec<_> = original_full.difference(&current_full).cloned().collect();

        let current_string: Vec<String> = current_diff.iter().map(|s| s.to_string()).collect();
        let original_string: Vec<String> = original_diff.iter().map(|s| s.to_string()).collect();
        if self.diff {
            println!("Saving current directory list to diff");
            //let _diff_file_write = filesystem::write_diff_file(&current_files, &diff_file_path);
        }
        (current_string, original_string)
    }
}

fn get_time() -> f64 {
    let systime = SystemTime::now();
    let unix_time = systime.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    (unix_time / 60.0 / 60.0 / 24.0).floor()
}

fn build_state() -> Result<State, Error> {
    let home = var("HOME").unwrap();
    let cache_home = var("XDG_CACHE_HOME").unwrap_or(format!("{}/.cache", &home));
    let default_directory = String::from(&home);
    let default_cache = format!("{}/lsdiff.list", &cache_home);

    let mut directory = var("LSDIFF_DIR").unwrap_or(String::from(&home));
    let cache = var("LSDIFF_CACHE").unwrap_or(default_cache);

    let folder_icon = var("LSDIFF_ICON_FOLDER").unwrap_or(String::from(""));
    let file_icon = var("LSDIFF_ICON_FILE").unwrap_or(String::from(""));

    if !Path::new(&directory).exists() {
        println!(
            "path literal '{}' does not exist! Falling back to {}",
            &directory, &default_directory
        );
        directory = default_directory
    }
    if !Path::new(&cache).exists() {
        println!("path literal '{}' does not exist! Creating...", &cache,);
    }

    let forcediff = parse_args(&directory, &cache, &folder_icon, &file_icon);

    let mut state = State {
        path: directory,
        path_list: Vec::new(),
        cache: cache,
        cache_list: Vec::new(),
        icon_folder: folder_icon,
        icon_file: file_icon,
        diff: forcediff,
        time: get_time(),
    };
    state.get_diff_state()?;
    state.get_path_state()?;

    Ok(state)
}

fn main() -> Result<(), Error> {
    let state = build_state()?;
    println!("{}", state.path_list.join(" "));
    let (path, diff) = state.compare_lists();
    output(&state, path, diff);

    Ok(())
}

fn output(state: &State, current: Vec<String>, diff: Vec<String>) -> () {
    let lscolors = LsColors::from_env().unwrap_or_default();

    let output_format = |file: String, op: String| {
        let path_str = format!("{}/{}", &state.path, &file);
        let path = Path::new(&path_str);
        let style = lscolors.style_for_path(&path_str);

        let ansi_style = style.map(Style::to_nu_ansi_term_style).unwrap_or_default();

        let mut output_str = file;
        if path.is_dir() {
            output_str = format!("{} {}", &state.icon_folder, output_str)
        } else {
            output_str = format!("{} {}", &state.icon_file, output_str)
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
    for file in diff {
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

fn parse_args(
    filepath: &str,
    diff_file_path_str: &str,
    folder_icon: &str,
    file_icon: &str,
) -> bool {
    let mut res = false;
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
    } else if first_arg.contains("-u") {
        res = true
    }
    res
}
