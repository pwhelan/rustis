use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::collections::HashMap;

fn split_command(buf: &str) -> Vec<&str> {
	buf.split_whitespace().clone().collect()
}

enum HandleConnectionError {
	UTF8Error(std::string::FromUtf8Error),
	IOError(std::io::Error),
}

impl std::fmt::Display for HandleConnectionError {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HandleConnectionError::IOError(err) => {
				fmt.write_str(&err.to_string())
			},
			HandleConnectionError::UTF8Error(err) => {
				fmt.write_str(&err.to_string())			
			}
		}
	}
}

fn handle_connection(cache: &mut HashMap<String, String>, mut stream: TcpStream) -> Result<usize, HandleConnectionError> {
	let mut buf: Vec<u8> = vec![0; 1024];
	match stream.read(&mut buf) {
		Ok(_) => {},
		Err(err) => {
			return Err(HandleConnectionError::IOError(err))
		}
	};

	match String::from_utf8(buf) {
		Ok(commands) => {
			let command_tokens = split_command(&commands);

			if command_tokens[0] == "GET" {
				match cache.get(command_tokens[1]) {
					Some(value) => {
						let response = format!("FOUND:{}", value);
						match stream.write(response.as_bytes()) {
							Ok(_) => {},
							Err(err) => {
								return Err(HandleConnectionError::IOError(err));
							}
						};
					},
					None => {
						match stream.write("NOT_FOUND".as_bytes()) {
							Ok(_) => {},
							Err(err) => {
								return Err(HandleConnectionError::IOError(err));
							}
						};
					},
				};
			} else if command_tokens[0] == "SET" {
				let key = command_tokens[1].to_owned();
				let val = command_tokens[2].to_owned();

				cache.insert(key, val);

				match stream.write("OK".as_bytes()) {
					Ok(_) => {},
					Err(err) => {
						return Err(HandleConnectionError::IOError(err));
					}
				};
			} else {
				match stream.write("ERR:BAD_COMMAND".as_bytes()) {
					Ok(_) => {},
					Err(err) => {
						return Err(HandleConnectionError::IOError(err));
					}
				};
			}
		},
		Err(err) => {
			return Err(HandleConnectionError::UTF8Error(err))
		},
	};

	Ok(1)
}

fn main() -> std::io::Result<()> {
	let listener = TcpListener::bind("127.0.0.1:35545")?;
	let mut cache: HashMap<String, String> = HashMap::new();


	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				match handle_connection(&mut cache, stream) {
					Ok(_) => {},
					Err(err) => {
						panic!("Error: {}", err)
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