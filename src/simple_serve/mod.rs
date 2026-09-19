/**
 * **注意**
 * 不论何种原因，此文件除了用户，禁止修改，禁止任何编辑
 */

pub use simple_tauri_macros::set_start_cmd;

mod work_dir;
mod local_ver;
mod spawn;
mod status;
mod action;
mod update;
mod log;
mod depsenv;

pub use work_dir::{
    set_work_dir,
    get_work_dir};
pub use local_ver::{
    set_local_ver,
    get_local_ver,
    check_local_ver};
pub use spawn::{
    spawn_in_dir};
pub use status::{
    is_running,
    IntoPort,
    wait_port,
    wait_port_timeout};
pub use action::{
    set_start_cmd,
    start,
    stop,
    child};
pub use update::{
    set_pkg,
    get_latest_ver,
    auto_check_update,
    check_update,
    set_download_url,
    set_ensure_server};
pub use log::{
    set_log_path,
    get_log_path,
    get_log};
pub use depsenv::{
    set_depsenv};
