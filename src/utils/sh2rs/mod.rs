mod wget;
mod mkdir;
mod rm;
mod unzip;
mod sleep;

pub use wget::*;
pub use mkdir::*;
pub use rm::*;
pub use unzip::*;
pub use sleep::*;

pub fn sh2rs(cmd: &str) -> Result<(), String> {
    let trimmed = cmd.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() {
        return Err(format!("unsupported command: {cmd}"));
    }
    match parts[0] {
        "wget" => {
            if parts.get(1) == Some(&"-O") {
                if parts.get(2) == Some(&"-") {
                    wget_O_(&parts[3..].join(" "))
                } else if parts.get(2) == Some(&"/dev/null") {
                    // 丢弃body
                    Ok(())
                } else {
                    wget_O(parts[2], &parts[3..].join(" "))
                }
            } else {
                wget(&parts[1..].join(" "))
            }
        }
        "mkdir" => {
            if parts.get(1) == Some(&"-p") {
                mkdir_p(&parts[2..].join(" "))
            } else {
                mkdir(&parts[1..].join(" "))
            }
        }
        "rm" => {
            if parts.get(1) == Some(&"-rf") {
                rm_r(&parts[2..].join(" "))
            } else {
                rm(&parts[1..].join(" "))
            }
        }
        "unzip" => {
            if parts.len() >= 4 {
                unzip(parts[1], parts[2], parts[3])
            } else if parts.len() == 3 {
                unzip(parts[1], parts[2], "")
            } else {
                Err("用法: unzip <file> <dir> [extract_dir]".into())
            }
        }
        "sleep" => {
            sleep(&parts[1..].join(" "))
        }
        _ => Err(format!("unsupported command: {}", parts[0])),
    }
}

pub fn sh2rs_rst(cmd: &str,_type: &str) -> String {
    if cmd=="wget" {
        get_wget_result(_type)
    } else{
        String::new()
    }
}