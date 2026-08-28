use simple_tauri::utils::{
    append_redirection, error_redirection, get_stderr_buffer, get_stdin_buffer, get_stdout_buffer,
    in_redirection, out_redirection, set_stderr_buffer, set_stdout_buffer,
};
use std::fs;

#[test]
fn redirection_works() {
    let dir = std::env::temp_dir();
    let out = dir.join("st_out.txt");
    let ap = dir.join("st_append.txt");
    let er = dir.join("st_err.txt");
    let in_f = dir.join("st_in.txt");

    // 输出重定向 >：已存在覆盖
    let _ = fs::remove_file(&out);
    fs::write(&out, "old").unwrap();
    set_stdout_buffer("new");
    out_redirection(">", out.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&out).unwrap(), "new");

    // 输出重定向 >：新文件创建
    let _ = fs::remove_file(&out);
    assert!(!out.exists());
    set_stdout_buffer("hello-out");
    out_redirection(">", out.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&out).unwrap(), "hello-out");
    assert_eq!(get_stdout_buffer(), "");

    // 输出重定向 >：&N 文件描述符 → 未实现
    set_stdout_buffer("x");
    assert!(out_redirection(">", "&1").is_err());

    // 输出重定向 >：/dev/null 丢弃
    set_stdout_buffer("drop-me");
    out_redirection(">", "/dev/null").unwrap();
    assert_eq!(get_stdout_buffer(), "");

    // 输出重定向 >：目录 → 未实现（非文件不能随便写）
    set_stdout_buffer("x");
    assert!(out_redirection(">", dir.to_str().unwrap()).is_err());

    // 输出重定向 >：特殊文件系统（/dev/tty、/proc）不允许任意写 → 未实现
    set_stdout_buffer("x");
    assert!(out_redirection(">", "/dev/tty").is_err());
    assert!(out_redirection(">", "/proc/self/mem").is_err());
    let _ = fs::remove_file(&out);

    // 追加重定向 >>：已存在追加
    let _ = fs::remove_file(&ap);
    fs::write(&ap, "a").unwrap();
    set_stdout_buffer("b");
    append_redirection(">>", ap.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&ap).unwrap(), "ab");

    // 追加重定向 >>：新文件创建
    let _ = fs::remove_file(&ap);
    assert!(!ap.exists());
    set_stdout_buffer("b");
    append_redirection(">>", ap.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&ap).unwrap(), "b");

    // 追加重定向 >>：&N → 未实现
    set_stdout_buffer("x");
    assert!(append_redirection(">>", "&2").is_err());
    let _ = fs::remove_file(&ap);

    // 错误重定向 2>：已存在覆盖
    let _ = fs::remove_file(&er);
    fs::write(&er, "old").unwrap();
    set_stderr_buffer("boom");
    error_redirection("2>", er.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&er).unwrap(), "boom");

    // 错误重定向 2>：新文件创建
    let _ = fs::remove_file(&er);
    assert!(!er.exists());
    set_stderr_buffer("boom");
    error_redirection("2>", er.to_str().unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&er).unwrap(), "boom");
    assert_eq!(get_stderr_buffer(), "");

    // 错误重定向 2>：/dev/null 丢弃
    set_stderr_buffer("drop-me");
    error_redirection("2>", "/dev/null").unwrap();
    assert_eq!(get_stderr_buffer(), "");
    let _ = fs::remove_file(&er);

    // 输入重定向 <：文件读入 stdin
    let _ = fs::remove_file(&in_f);
    fs::write(&in_f, "input-data").unwrap();
    in_redirection("<", in_f.to_str().unwrap()).unwrap();
    assert_eq!(get_stdin_buffer(), "input-data");

    // 输入重定向 <：&N → 未实现
    assert!(in_redirection("<", "&0").is_err());

    // 输入重定向 <：/dev/null 空输入
    in_redirection("<", "/dev/null").unwrap();
    assert_eq!(get_stdin_buffer(), "");
    let _ = fs::remove_file(&in_f);
}
