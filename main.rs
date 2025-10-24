use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use tracing::{event, span, Level};

mod dispatch;

fn split_command(buf: &str) -> Vec<&str> {
	buf.split_whitespace().clone().collect()
}

fn handle_connection(
	cache_lock: &RwLock<HashMap<String, String>>,
	mut stream: TcpStream
) -> Result<dispatch::Status, Box<dyn std::error::Error>>
{
	let addr = stream.peer_addr()?;
	let span = span!(Level::INFO, "handle_connection", client=format!("{}", addr));
	let _guard = span.enter();

	event!(Level::INFO, "handle_connection");

	loop {
		let mut buf: Vec<u8> = vec![0; 1024];
		let blen = stream.read(&mut buf)?;
		let input: Vec<u8> = buf[0..blen].to_vec();

		match String::from_utf8(input) {
			Ok(commands) => {
				let command_tokens = split_command(&commands);
				event!(Level::DEBUG, commands=commands, tokens=command_tokens.join(","));

				let response = dispatch::dispatch_command(cache_lock, &mut stream,
					command_tokens)?;

				if response == dispatch::Status::Quit {
					event!(Level::INFO, "handle_connection: exit");
					return Ok(response)
				}
			},
			Err(err) => {
				event!(Level::ERROR, error=format!("{}", err));
				return Err(Box::new(err))
			},
		};
	}
}

fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt::init();

	let listener = TcpListener::bind("127.0.0.1:35545")?;
	event!(Level::INFO, "listening on 127.0.0.1:35545");
	
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
							event!(Level::ERROR, error=format!("{}", err));
						}
					}
				});
			}
			Err(err) => {
				event!(Level::ERROR, error=format!("{}", err));
				return Err(err)
			}
		}
	}
	Ok(())
}
