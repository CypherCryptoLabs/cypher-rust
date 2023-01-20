# Test registering a Node
```curl -X POST -d "{\"ip_address\": \"123.123.123.123\", \"blockchain_address\": \"0x0000000000000000000000000000000000000000\", \"registration_timestamp\": $(date +%s%3N)}" localhost:1234/v0.0.1/network/node```

# Test if Node was added
```curl localhost:1234/v0.0.1/network```