use simple_tauri::simple_serve::get_log_path;

/// get_log_path 首次调用（PATH 为 None）不得死锁：
/// 旧实现在持锁状态下调用 set_log_path，同线程二次加锁导致永久挂起。
#[test]
fn get_log_path_no_deadlock_on_first_call() {
    let p1 = get_log_path();
    let p2 = get_log_path();
    assert_eq!(p1, p2);
    assert!(p1.is_absolute(), "fallback 应返回绝对路径: {p1:?}");
}
