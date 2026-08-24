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
    wait_port,
    wait_port_timeout};
pub use action::{
    set_start_cmd,
    start,
    stop,
    child};
