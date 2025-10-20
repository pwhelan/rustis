use std::net::TcpStream;
use std::io::Write;
use std::collections::HashMap;
use std::sync::RwLock;

pub fn dispatch_command(cache_lock: &RwLock<HashMap<String, String>>, stream: &mut TcpStream, command_tokens: Vec<&str>) -> Result<usize, Box<dyn std::error::Error>> {
	if command_tokens[0] == "GET" {
		if command_tokens.len() < 2 {
			stream.write("ERR:MISSING_ARGUMENTS\n".as_bytes())?;
			return Ok(1)
		}
		match cache_lock.read() {
			Ok(cache) => {
				match cache.get(command_tokens[1]) {
					Some(value) => {
						let response = format!("OK:{}\n", value);
						stream.write(response.as_bytes())?;
					},
					None => {
						stream.write("ERR:NOT_FOUND\n".as_bytes())?;
					},
				};
			},
			Err(_) => {
				panic!("CacheLock has been poisoned");
			}
		}
	} else if command_tokens[0] == "SET" {
		if command_tokens.len() < 2 {
			stream.write("ERR:MISSING_ARGUMENTS\n".as_bytes())?;
			return Ok(1)
		} 
		match cache_lock.write() {
			Ok(mut cache) => {
				let key = command_tokens[1].to_owned();
				let val = command_tokens[2].to_owned();

				cache.insert(key, val);

				stream.write("OK\n".as_bytes())?;
			},
			Err(_) => {
				panic!("CacheLock has been poisoned");
			}
		}
		
	} else if command_tokens[0] == "QUIT" {
		return Ok(1);
	} else {
		stream.write("ERR:BAD_COMMAND\n".as_bytes())?;
	}
	Ok(1)
}
