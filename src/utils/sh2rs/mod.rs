mod wget;
mod mkdir;
mod rm;
mod unzip;

pub use wget::*;
pub use mkdir::*;
pub use rm::*;
pub use unzip::*;

pub fn sh2rs(cmd: &str) -> Result<(), String> {
	let trimmed = cmd.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() {
        return Err(format!("unsupported command: {cmd}"));
    }
    match parts[0] {
        "wget"  => {
        	if parts[1]=="-O" {
        		wget_O(&parts[2],&parts[3..])
        	}else{
        		wget(&parts[1..])
        	}
        }
        "mkdir" => {
        	if parts[1]=="-p" {
        		mkdir_p(&parts[2..])
        	}else{
        		mkdir(&parts[1..])
        	}
        }
        "rm"    => {
        	if parts[1]=="-rf" {
        		rm_r(&parts[2..])
        	}else{
        		rm(&parts[1..]),
        	}
        }
        "unzip" => unzip(&parts[1..]),
        _ => Err(format!("unsupported command: {}", parts[0])),
    }
}