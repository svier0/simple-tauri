use simple_tauri::utils::{cat, echo, get_stdout_buffer};

#[test]
fn echo_writes_buffer() {
    echo("hello").unwrap();
    assert_eq!(get_stdout_buffer(), "hello");
}

#[test]
fn cat_reads_file() {
    let p = std::env::temp_dir().join("simple_tauri_test_cat.txt");
    std::fs::write(&p, "file-content").unwrap();
    cat(p.to_str().unwrap()).unwrap();
    assert_eq!(get_stdout_buffer(), "file-content");
    let _ = std::fs::remove_file(&p);
}
