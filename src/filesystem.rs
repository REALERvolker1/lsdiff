use std::{
    env::var,
    fs,
    io::Error,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};



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
