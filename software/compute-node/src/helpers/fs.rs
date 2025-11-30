use std::fs;
use std::io;
use std::path::Path;

fn clean_directory(target_dir: &Path) -> io::Result<()> {
    if target_dir.exists() {
        for entry in fs::read_dir(target_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        fs::create_dir_all(target_dir)?;
    }
    Ok(())
}

fn copy_recursively(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        let dest_path = destination.join(path.file_name().unwrap());

        if path.is_dir() {
            copy_recursively(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn sync_folders(source_dir: &Path, target_dir: &Path) -> io::Result<()> {
    if !source_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory not found: {:?}", source_dir),
        ));
    }

    clean_directory(target_dir)?;
    copy_recursively(source_dir, target_dir)?;

    Ok(())
}
