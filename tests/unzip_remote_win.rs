use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::thread;

fn serve(bytes: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        if let Ok((mut s, _)) = listener.accept() {
            // 先读掉客户端请求，否则关闭时未读数据会触发 RST
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                bytes.len()
            );
            let _ = s.write_all(resp.as_bytes());
            let _ = s.write_all(&bytes);
        }
    });
    format!("http://{addr}/node.zip")
}

fn make_zip() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut w = Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(&mut w);
        zip.start_file(
            "node-vTEST-win-x64/test.txt",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(b"simple-tauri").unwrap();
        zip.finish().unwrap();
    }
    buf
}

// 回归测试：sh2rs 解析器必须把反斜杠当字面量，Windows 路径分隔符不能丢。
// 直接走库内部 unzip_remote（demo 经 ensure_node 触发的同一路径），
// 证明 C:\...\node.zip 这类自动生成的路径能正确落盘。
#[test]
fn unzip_remote_keeps_backslash_path() {
    let base = std::env::temp_dir().join("simple_tauri_unzip_test");
    let base_str = base.to_string_lossy().into_owned();
    let _ = std::fs::remove_dir_all(&base_str);

    #[cfg(windows)]
    assert!(base_str.contains('\\'), "提取目录应含反斜杠: {base_str}");

    let url = serve(make_zip());
    simple_tauri::utils::unzip_remote(&url, &base_str, "node-vTEST-win-x64")
        .expect("unzip_remote 应成功提取");

    let out = base.join("test.txt");
    let content = std::fs::read_to_string(&out).expect("提取文件应存在");
    assert_eq!(content, "simple-tauri");

    let _ = std::fs::remove_dir_all(&base_str);
}
