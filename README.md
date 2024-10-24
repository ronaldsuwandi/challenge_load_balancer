# Load Balancer

Attempt to write a simple round-robin load balancer in Rust (compiled using Rust 1.81)

See [CHALLENGE.md](CHALLENGE.md) or
the [original challenge link](https://codingchallenges.fyi/challenges/challenge-load-balancer/)
for more details

Uses Tokio to handle the the request in async fashion. Currently it forwards
the request from the input socket straight to the target socket and write
the response directly (without buffering it in memory) for efficiency

I also added [gigachad.jpg](script/server1/gigachad.jpg) file in the server to handle transferring of larger
file - to ensure that the load balancer still work with file larger than
the specified buffer (4kb)

## To build

```
cargo build --release
```

Default binary output will be stored in `./target/release/challenge_load_balancer`

To execute

```
target/release/challenge_load_balancer config.toml
```

## To run servers

```
script/servers.sh
```

This will launch 3 servers serving script/server1, script/server2, script/server3

## To run load balancer in dev mode

```
cargo run <config.toml>
```

If config is not provided, it defaults to `./config.toml`

## To test

```
curl localhost:8080
```

OR browse to http://localhost:8080
