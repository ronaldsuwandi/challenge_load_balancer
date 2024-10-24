# Load Balancer

Attempt to write simple round-robin load balancer in Rust

## To run servers

```
script/servers.sh
```

This will launch 3 servers serving script/server1, script/server2, script/server3

## To run load balancer

```
cargo run <config.toml>
```

If config is not provided, it defaults to `./config.toml`

## To test

```
curl localhost:8080
```

OR browse to http://localhost:8080

## Todo

- [ ] better error handling
- [ ] handle await
- [x] healthcheck
- [ ] update build/run process
- [x] read config for lb
- [x] script to run server