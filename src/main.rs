use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write, Read};
use std::path::{Path, PathBuf};
use std::env;
use encoding_rs::ISO_8859_10; // Usando ISO-8859-10

fn list_dxf_files(dir_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut dxf_files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension.to_string_lossy().to_lowercase() == "dxf" {
                println!("File: {}", path.display());
                dxf_files.push(path);
            }
        }
    }
    Ok(dxf_files)
}

fn extract_numbers_from_names(paths: Vec<PathBuf>) -> Vec<(PathBuf, Vec<u32>)> {
    let mut sheet_list = Vec::new();
    for path in paths {
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy().to_string();
            let mut numbers = Vec::new();
            let mut current_number = String::new();
            for c in name.chars() {
                if c.is_digit(10) {
                    current_number.push(c);
                } else {
                    if !current_number.is_empty() {
                        if let Ok(num) = current_number.parse::<u32>() {
                            numbers.push(num);
                        }
                        current_number.clear();
                    }
                }
            }
            if !current_number.is_empty() {
                if let Ok(num) = current_number.parse::<u32>() {
                    numbers.push(num);
                }
            }
            sheet_list.push((path.clone(), numbers));
        }
    }
    sheet_list
}

fn update_files_with_sheet_number(paths: Vec<PathBuf>, sheet_list: Vec<(PathBuf, Vec<u32>)>) -> io::Result<()> {
    let total_files = paths.len();

    for (index, (path, _)) in sheet_list.iter().enumerate() {
        let sheet_number = index + 1;
        let new_content = format!("{}/{}", sheet_number, total_files);

        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Decodificando usando ISO-8859-10
        let (decoded, _, had_errors) = ISO_8859_10.decode(&data);
        if had_errors {
            eprintln!("Aviso: Erros de decodificação encontrados.");
        }

        let modified_data: String = decoded.lines()
            .map(|line| line.replace("XX/XX", &new_content))
            .collect::<Vec<_>>()
            .join("\n");

        // Convertendo de volta para bytes
        let (modified_bytes, _, _) = ISO_8859_10.encode(&modified_data);

        let mut file = BufWriter::new(OpenOptions::new().write(true).truncate(true).open(path)?);
        file.write_all(&modified_bytes)?;

    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dir_path = if args.len() > 1 {
        Path::new(&args[1]).to_path_buf()
    } else {
        env::current_dir()?
    };
    let dxf_files = list_dxf_files(&dir_path)?;
    let sheet_list = extract_numbers_from_names(dxf_files.clone());

    for (path, numbers) in &sheet_list {
        println!("File: {}, Numbers: {:?}", path.display(), numbers);
    }

    update_files_with_sheet_number(dxf_files, sheet_list)?;

    Ok(())
}
