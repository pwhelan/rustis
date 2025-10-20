# Rustis

A simple in-memory key-value store, inspired by Redis, written in Rust. README
written with the help of the X1 Robot, all code rolled by hand.

This is my initial project in rust to come to grips with the borrow checker,
threading and networking.

I also intend to add metrics to be able to observe how best to implement
locking with the internal datascruture that holds all the data to reduce
latency and locking contention.

## Features

*   **GET `<key>`**: Retrieve the value for a given key.
*   **SET `<key>` `<value>`**: Set a value for a given key.
*   **QUIT**: Close the connection.

## How to Run

1.  Build the project:
    ```bash
    cargo build
    ```
2.  Run the server:
    ```bash
    cargo run
    ```
3.  Connect to the server using a tool like `netcat`:
    ```bash
    nc 127.0.0.1 35545
    ```

## Example Usage

```
SET mykey myvalue
OK
GET mykey
OK:myvalue
QUIT
```
