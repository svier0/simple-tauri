mod operator;
mod wget;
mod mkdir;
mod rm;
mod unzip;
mod sleep;
mod cp;
mod mv;

pub use operator::*;
pub use wget::*;
pub use mkdir::*;
pub use rm::*;
pub use unzip::*;
pub use sleep::*;
pub use cp::*;
pub use mv::*;

pub fn sh2rs(input: &str) -> Result<(), String> {
    let parts: Vec<String> = shlex::split(input).ok_or("unterminated quote")?;

    if parts.is_empty() {
        return Err(format!("unsupported command: {input}"));
    }
    match parts[0].as_str() {
        "wget" => {
            if parts.get(1).map(String::as_str) == Some("-O") {
                if parts.get(2).map(String::as_str) == Some("-") {
                    wget_O_(&parts[3..].join(" "))
                } else if parts.get(2).map(String::as_str) == Some("/dev/null") {
                    // 丢弃body
                    Ok(())
                } else {
                    wget_O(&parts[2], &parts[3..].join(" "))
                }
            } else {
                wget(&parts[1..].join(" "))
            }
        }
        "sleep" => {
            sleep(&parts[1..].join(" "))
        }
        "mkdir" => {
            if parts.get(1).map(String::as_str) == Some("-p") {
                mkdir_p(&parts[2..].join(" "))
            } else {
                mkdir(&parts[1..].join(" "))
            }
        }
        "rm" => {
            if parts.get(1).map(String::as_str) == Some("-rf") {
                rm_r(&parts[2..].join(" "))
            } else {
                rm(&parts[1..].join(" "))
            }
        }
        "cp" => {
            cp(&parts[1], &parts[2])
        }
        "mv" => {
            mv(&parts[1], &parts[2])
        }
        "unzip" => {
            if parts.len() >= 4 {
                unzip(&parts[1], &parts[2], &parts[3])
            } else if parts.len() == 3 {
                unzip(&parts[1], &parts[2], "")
            } else {
                Err("用法: unzip <file> <dir> [extract_dir]".into())
            }
        }
        _ => Err(format!("unsupported command: {}", parts[0])),
    }
}