# Load Balancer

Attempt to write simple round-robin load balancer in Rust

## To run servers

```
python3 -m http.server 8081 --directory script/server1
python3 -m http.server 8082 --directory script/server2
python3 -m http.server 8083 --directory script/server3
```

## To run load balancer

```
cargo run
```

## To test

```
curl localhost:8080
```

OR browse to http://localhost:8080

## Todo

- [ ] better error handling
- [ ] handle await
- [ ] healthcheck
- [ ] update build/run process
- [ ] read config for lb