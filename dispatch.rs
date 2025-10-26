use std::net::TcpStream;
use std::io::Write;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{event, Level};

#[derive(PartialEq)]
pub enum Status {
	Error,
	Ok,
	Quit,
}

struct Response {
	status: Status,
	payload: String,
}

struct Command {
	verb: String,
	arguments: usize,
	function: fn(&RwLock<HashMap<String, String>>, command_tokens: Vec<&str>)
		-> Result<Response, Box<dyn std::error::Error>>,
}

fn execute_get(
	cache_lock: &RwLock<HashMap<String, String>>,
	command_tokens: Vec<&str>
) -> Result<Response, Box<dyn std::error::Error>>
{
	match cache_lock.read() {
		Ok(cache) => {
			match cache.get(command_tokens[1]) {
				Some(value) => {
					return Ok(Response{
						status: Status::Ok,
						payload: format!("OK:{}\n", value),
					});
				},
				None => {
					return Ok(Response{
						status: Status::Error,
						payload: String::from("ERR:NOT_FOUND\n"),
					});
				},
			};
		},
		Err(_) => {
			panic!("CacheLock has been poisoned");
		}
	};
}

fn execute_set(
	cache_lock: &RwLock<HashMap<String, String>>,
	command_tokens: Vec<&str>
)  -> Result<Response, Box<dyn std::error::Error>>
{
	match cache_lock.write() {
		Ok(mut cache) => {
			let key = command_tokens[1].to_owned();
			let val = command_tokens[2].to_owned();

			cache.insert(key, val);
			return Ok(Response{
				status: Status::Ok,
				payload: String::from("OK\n"),
			});
		},
		Err(_) => {
			panic!("CacheLock has been poisoned");
		}
	}
}

fn execute_quit(
	_cache_lock: &RwLock<HashMap<String, String>>,
	_command_tokens: Vec<&str>
) -> Result<Response, Box<dyn std::error::Error>>
{
	return Ok(Response{
		status: Status::Quit,
		payload: String::from("OK\n"),
	})
}

pub fn dispatch_command(
	cache_lock: &RwLock<HashMap<String, String>>,
	stream: &mut TcpStream,
	command_tokens: Vec<&str>
) -> Result<Status, Box<dyn std::error::Error>> 
{
	let commands = vec![
		Command { 
			verb: String::from("GET"),
			arguments: 1,
			function: execute_get,
		},
		Command {
			verb: String::from("SET"),
			arguments: 2,
			function: execute_set,
		},
		Command {
			verb: String::from("QUIT"),
			arguments: 0,
			function: execute_quit,
		}
	];

	if command_tokens.is_empty() {
		let _rc = stream.write("ERR:EMPTY_COMMAND\n".as_bytes())?;
		return Ok(Status::Error);
	}

	for command in commands {
		if command_tokens[0] == command.verb {

			event!(Level::DEBUG, verb = command.verb);

			if command_tokens.len()-1 < command.arguments {
				event!(Level::ERROR, msg="missing arguments",
				passed=command_tokens.len()-1, wanted=command.arguments);
				let _rc = stream.write("ERR:MISSING_ARGUMENTS\n".as_bytes())?;
				return Ok(Status::Error)
			}

			let resp = (command.function)(cache_lock, command_tokens)?;
			let _rc = stream.write(resp.payload.as_bytes())?;

			event!(Level::DEBUG, response=resp.payload);

			return Ok(resp.status)
		}
	}

	let _rc = stream.write("ERR:BAD_COMMAND\n".as_bytes())?;
	Ok(Status::Error)
}
