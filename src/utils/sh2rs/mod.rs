pub use simple_tauri_macros::sh2rs;
pub use simple_tauri_macros::try_quote;

mod operator;
mod cd;
mod sh;
mod echo;
mod cat;
mod sleep;
mod wget;
mod mkdir;
mod rm;
mod unzip;
mod cp;
mod mv;

pub use operator::*;
pub use cd::*;
pub use sh::*;
pub use echo::*;
pub use cat::*;
pub use sleep::*;
pub use wget::*;
pub use mkdir::*;
pub use rm::*;
pub use unzip::*;
pub use cp::*;
pub use mv::*;

pub fn sh2rs(input: &str) -> Result<(), String> {
    eprintln!("debug: sh2rs: {input}");

    let parts: Vec<String> = shlex::split(input).ok_or("unterminated quote")?;

    if parts.is_empty() {
        return Err(format!("unsupported command: {input}"));
    }

    // 判断重定向操作符：找到最后一个重定向符号
    if let Some((op_idx, op)) = parts
        .iter()
        .enumerate()
        .rev()
        .find(|(_, t)| matches!(t.as_str(), "<" | ">" | ">>" | "2>" | "2>>"))
    {
        let target = parts
            .get(op_idx + 1)
            .ok_or_else(|| format!("重定向缺少目标: {}", op))?;
        let left = parts[..op_idx].iter().map(|t| if t.contains(' ') { try_quote(t) } else { t.to_string() }).collect::<Vec<_>>().join(" ");
        return match op.as_str() {
            "<"   => in_redirection(&left, target),
            ">"   => out_redirection(&left, target),
            ">>"  => append_redirection(&left, target),
            "2>"  => error_redirection(&left, target),
            "2>>" => error_append_redirection(&left, target),
            _     => unreachable!(),
        };
    }

    // 分流 执行具体命令
    match parts[0].as_str() {
        "cd" => {
            cd(&parts[1..].join(" "))
        }
        "sh" => {
            sh(&parts[1..].join(" "))
        }
        "echo" => {
            echo(&parts[1..].join(" "))
        }
        "cat" => {
            cat(&parts[1..].join(" "))
        }
        "sleep" => {
            sleep(&parts[1..].join(" "))
        }
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

pub fn try_quote(input: &str) -> String {
    shlex::try_quote(input)
        .unwrap_or_default()
        .to_string()
}