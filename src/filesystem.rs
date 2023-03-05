use std::{
    env::var,
    fs,
    io::Error,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct State {
    home_folder: String,
    path: String,
    cache: String,
    icon_folder: String,
    icon_file: String,
}
// let home = var("HOME").unwrap();
impl State {
    fn read_normal_dir(&self) -> Result<Vec<String>, Error> {
        let files = fs::read_dir(self.path)?;

        let current_files: Vec<String> = Vec::new();
        for file_entry in files {
            let file = file_entry?.file_name();
            current_files.push(format!("{}", file.to_str().unwrap())); // potential point of failure
        }
        Ok(current_files)
    }
}

pub fn build_state() -> Result<State, Error> {
    let home = var("HOME").unwrap();
    let cache_home = var("XDG_CACHE_HOME").unwrap_or(format!("{}/.cache", &home));
    let directory = var("LSDIFF_DIR").unwrap_or(home);
    let diff_file_path_str = var("LSDIFF_CACHE").unwrap_or(format!("{}/lsdiff.list", cache_home));
    let folder_icon = var("LSDIFF_ICON_FOLDER").unwrap_or(String::from(""));
    let file_icon = var("LSDIFF_ICON_FILE").unwrap_or(String::from(""));

    Ok(State {
        home_folder: home,
        path: directory,
        cache: diff_file_path_str,
        icon_folder: folder_icon,
        icon_file: file_icon,
    })
}

pub fn read_diff_file(diff_file: &Path, arg: &str) -> Result<(Vec<String>, bool), Error> {
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
    if file_time != unix_time_days || arg.contains("-u") {
        diff = true
    }
    Ok((files, diff))
}

pub fn write_diff_file(current_files: &Vec<String>, diff_file_path: &Path) -> Result<(), Error> {
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

pub fn read_normal_dir(folder: &str) -> Result<Vec<String>, Error> {
    let files = fs::read_dir(folder)?;

    let mut current_files: Vec<String> = Vec::new();

    for file_entry in files {
        let file = file_entry?.file_name();
        current_files.push(format!("{}", file.to_str().unwrap())); // potential point of failure
    }
    Ok(current_files)
}
