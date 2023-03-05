use lscolors::{LsColors, Style};
use nu_ansi_term::Color;

pub fn output(basepath: &str, icons: &Icon, current: Vec<String>, original: Vec<String>) -> () {
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
