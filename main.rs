use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

mod dispatch;

fn split_command(buf: &str) -> Vec<&str> {
	buf.split_whitespace().clone().collect()
}

fn handle_connection(
	cache_lock: &RwLock<HashMap<String, String>>,
	mut stream: TcpStream
) -> Result<dispatch::Status, Box<dyn std::error::Error>>
{
	loop {
		let mut buf: Vec<u8> = vec![0; 1024];
		stream.read(&mut buf)?;

		match String::from_utf8(buf) {
			Ok(commands) => {
				let command_tokens = split_command(&commands);
				let response = dispatch::dispatch_command(cache_lock, &mut stream,
					command_tokens)?;

				if response == dispatch::Status::Quit {
					return Ok(response)
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
	let cache_arc = Arc::new(cache_lock);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let cache_lock_handle = Arc::clone(&cache_arc);
				thread::spawn(move || {
					match handle_connection(&cache_lock_handle, stream) {
						Ok(_) => {},
						Err(err) => {
							panic!("Stream Error: {}", err)
						}
					}
				});
			}
			Err(err) => {
				return Err(err)
			}
		}
	}
	Ok(())
}