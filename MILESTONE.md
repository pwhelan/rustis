# Rustis v1.0

## 1. Comprehensive Logging Infrastructure

Estimate: 1 day
Labels: ops, logging, v1.0
Priority: Critical

Objective: We should have a great, configurable logging system for tracking problems, seeing what the server is doing, and keeping a record of important events.

Sub-Tasks

- [ ] Integrate a modern, structured logging library (like tracing) and get it configured.
- [ ] Set up logging levels (Debug, Info, Warn, Error) that we can control easily, maybe with a config file or environment variables.
- [ ] Make sure we're logging key events, like:
  - [ ] Server starting and stopping.
  - [ ] Clients connecting and disconnecting.
  - [ ] Command execution (maybe only at the DEBUG level).
  - [ ] Errors and any warnings (especially if they involve concurrency or locking!).
- [ ] The log output needs to be clean and easy for machines to read (like JSON or key-value pairs).

## 2. Performance Counters and Lock Metrics

Estimate: 4 days
Labels: observability, monitoring, performance, v1.0
Priority: Critical

Objective: We need comprehensive metrics so we can see exactly what's happening inside the server, especially around locking! We want to spot any lock contention immediately.

Sub-Tasks

- [ ] Integrate a metrics library that works with tools like Prometheus.
- [ ] Instrument the code to track simple stuff like commands_processed, bytes_read, and bytes_written.
- [ ] Implement specific lock-related metrics:
  - [ ] How long are people waiting for a lock? (lock_wait_duration_seconds histogram).
  - [ ] How often do lock attempts fail? (lock_acquire_failures counter).
  - [ ] How many locks are currently active? (active_locks gauge).
- [ ] Set up a simple /metrics endpoint so external monitoring tools can scrape the data.

## 3. Implement REDIS-like Text Protocol (RESP)

Estimate: 1 week
Labels: protocol, networking, v1.0
Priority: Critical

Objective: We need a solid parser and serializer for the Redis Serialization Protocol (RESP) so all the standard Redis clients can talk to our server easily. Compatibility is key!

Sub-Tasks

- [ ] Read and propose a subset of RESP to implement
  - [ ] Define the subset of VERBS to implement.
  - [ ] Define the data types for RESP (Simple Strings, Errors, Integers, Bulk Strings, Arrays).
- [ ] Implement a fast, non-blocking RESP parser that handles partial socket reads gracefully.
- [ ] Implement the RESP serializer.
- [ ] Implement missing VERBS.

## 4. Key-Level Locking Mechanism

Estimate: 1 week
Labels: concurrency, data-store, v1.0
Priority: High

Objective: We've gotta make sure data access is totally safe for multi-threading! We'll use smart, granular locking for each individual key instead of one huge, slow lock for the whole database. No global bottlenecks here!

Sub-Tasks

- [ ] Research and test key-level locking. Write up a brief describing how it works.
- [ ] Review other Rust concurrency patterns to make sure we're keeping lock times as short as possible.
- [ ] Develop some serious stress tests to confirm the safety and see how fast our key-level locking is when things get busy.

## 5. Implement Alternative Binary Protocol

Estimate: 1 week
Labels: protocol, performance, v1.0
Priority: Low (stretch goal)

Objective: Time to build our own super-efficient binary protocol! This one should be even faster than RESP, with way less overhead, for maximum performance.

Sub-Tasks

- [ ] Decide on the base binary format for the binary protocol
  - [ ] Msgpack
  - [ ] Protobuf
  - [ ] Other?
- [ ] Design the structure for this new binary protocol (like message headers and how data is encoded).
- [ ] Implement a dedicated parser and serializer just for this binary format.
- [ ] Add Configuration so that listeners can be define by address, port and protocol.
- [ ] Make sure to write some performance tests comparing how fast RESP and Binary serialization/deserialization really are!
