
use crate::simple_serve;
use crate::config;
use crate::simple_tray::ipc_result;
use crate::simple_tray::app;

// ------- ------- 本体版本与更新 ------- -------

/// 获取本体版本号
#[tauri::command]
pub fn ipc_version() -> serde_json::Value {
    let r = app().package_info().version.to_string();
    ipc_result!(0,"",r)
}

/// 获取最新版本号 TODO:
#[tauri::command]
pub fn _ipc_latest_ver() -> serde_json::Value {
	ipc_result!(1,"未实现")
}

/// 更新本体 TODO:
#[tauri::command]
pub fn _ipc_update() -> serde_json::Value {
	ipc_result!(1,"未实现")
}

// ------- ------- 服务版本与更新 ------- -------

/// 获取服务端本号
#[tauri::command]
pub fn ipc_server_version() -> serde_json::Value {
    let r = simple_serve::get_local_ver();
    ipc_result!(0,"",r)
}

/// 获取服务端最新版本号
#[tauri::command]
pub fn ipc_server_latest_ver() -> serde_json::Value {
    ipc_result!(simple_serve::get_latest_ver())
}

/// 更新服务端
#[tauri::command]
pub fn ipc_server_update() -> serde_json::Value {
    ipc_result!(simple_serve::check_update(config::get_or!("auto_update", false)))
}

// ------- ------- 配置读写 ------- -------

/// 获取配置列表
#[tauri::command]
pub fn ipc_config() -> serde_json::Value {
    ipc_result!(0,"",config::all())
}

/// 设置单个配置
#[tauri::command]
pub fn ipc_set_config(key: &str,val: serde_json::Value) -> serde_json::Value {
    ipc_result!(config::set(key,val))
}

/// 批量设置配置
#[tauri::command]
pub fn ipc_set_configs(configs: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(map) = configs {
        for (key, val) in map {
            let _ = config::set(&key, val);
        }
    }
    ipc_result!(0, "")
}

// ------- ------- 服务管理 ------- -------

/// 获取服务状态
#[tauri::command]
pub fn ipc_server_status() -> serde_json::Value {
    ipc_result!(0,"",simple_serve::is_running())
}

/// 设置服务状态
#[tauri::command]
pub fn ipc_server_action(action: &str) -> serde_json::Value {
    if action=="start" {
        let r = simple_serve::start()
            .map_err(|e| format!("服务器启动失败: {e}"));
        ipc_result!(r)
    }else{
        simple_serve::stop();
        ipc_result!(0,"")
    }
}

/// 读取服务日志
#[tauri::command]
pub fn ipc_server_logs() -> serde_json::Value {
	ipc_result!(0,"",simple_serve::get_log())
}
