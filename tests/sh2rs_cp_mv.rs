use simple_tauri::utils::{cp, mv};
use std::fs;
use std::path::Path;

fn tmp(name: &str) -> String {
    let p = std::env::temp_dir().join(format!("simple_tauri_test_{}", name));
    let _ = fs::remove_dir_all(&p);
    p.to_string()
}

#[test]
fn cp_file_and_dir() {
    let src = tmp("cp_src");
    fs::create_dir_all(&src).unwrap();
    fs::write(format!("{}/a.txt", src), "hello").unwrap();
    fs::create_dir_all(format!("{}/sub", src)).unwrap();
    fs::write(format!("{}/sub/b.txt", src), "world").unwrap();

    let dst = tmp("cp_dst");
    cp(&src, &dst).unwrap();
    assert!(Path::new(&format!("{}/a.txt", dst)).exists());
    assert!(Path::new(&format!("{}/sub/b.txt", dst)).exists());

    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&dst);
}

#[test]
fn cp_into_existing_dir() {
    let src = tmp("cp_into_src");
    fs::write(&src, "x").unwrap();
    let dst = tmp("cp_into_dst");
    fs::create_dir_all(&dst).unwrap();

    cp(&src, &dst).unwrap();
    let src_name = Path::new(&src).file_name().unwrap().to_string();
    assert!(Path::new(&dst).join(&src_name).exists());

    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&dst);
}

#[test]
fn mv_file() {
    let dir = tmp("mv_dir");
    fs::create_dir_all(&dir).unwrap();
    let f = format!("{}/x.txt", dir);
    fs::write(&f, "data").unwrap();
    let dst = tmp("mv_dst");
    fs::create_dir_all(&dst).unwrap();

    mv(&f, &dst).unwrap();
    assert!(Path::new(&format!("{}/x.txt", dst)).exists());
    assert!(!Path::new(&f).exists());

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&dst);
}

#[test]
fn cp_missing_source_errors() {
    let r = cp("/no/such/path/xyz", "/tmp/whatever");
    assert!(r.is_err());
}
