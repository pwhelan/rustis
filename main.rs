use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::collections::HashMap;
use std::sync::RwLock;

fn split_command(buf: &str) -> Vec<&str> {
	buf.split_whitespace().clone().collect()
}

fn handle_connection(cache_lock: &RwLock<HashMap<String, String>>, mut stream: TcpStream) -> Result<usize, Box<dyn std::error::Error>> {

	loop {
		let mut buf: Vec<u8> = vec![0; 1024];
		stream.read(&mut buf)?;

		match String::from_utf8(buf) {
			Ok(commands) => {
				let command_tokens = split_command(&commands);

				if command_tokens[0] == "GET" {
					match cache_lock.read() {
						Ok(cache) => {
							match cache.get(command_tokens[1]) {
								Some(value) => {
									let response = format!("OK:{}", value);
									stream.write(response.as_bytes())?;
								},
								None => {
									stream.write("ERR:NOT_FOUND".as_bytes())?;
								},
							};
						},
						Err(_) => {
							panic!("CacheLock has been poisoned");
						}
					}
				} else if command_tokens[0] == "SET" {
					match cache_lock.write() {
						Ok(mut cache) => {
							let key = command_tokens[1].to_owned();
							let val = command_tokens[2].to_owned();

							cache.insert(key, val);

							stream.write("OK".as_bytes())?;
						},
						Err(_) => {
							panic!("CacheLock has been poisoned");
						}
					}
				} else if command_tokens[0] == "QUIT" {
					return Ok(1);
				} else {
					stream.write("ERR:BAD_COMMAND".as_bytes())?;
				}
			},
			Err(err) => {
				return Err(Box::new(err))
			},
		};
	}

}

fn main() -> std::io::Result<()> {
	let listener = TcpListener::bind("127.0.0.1:35545")?;
	let cache: HashMap<String, String> = HashMap::new();
	let cache_lock = RwLock::new(cache);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				match handle_connection(&cache_lock, stream) {
					Ok(_) => {},
					Err(err) => {
						panic!("Stream Error: {}", err)
					}
				}
			}
			Err(err) => {
				return Err(err)
			}
		}
	}
	Ok(())
}