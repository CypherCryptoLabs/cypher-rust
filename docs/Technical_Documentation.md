# Cypher - Technical Documentation

## API
Cypher uses a REST API to communicate with wallets, as well as other Nodes. A REST API was chosen, because of its simplicity, and ease of development for all parties. Almost all modern applications use some kind of REST API, which means many developers have experience with it.

Cyphers API is accessible with the "1234" port. A HTTP server is listening to that port and serving the following endpoints:

| Path | Method |Information|
-------|--------|-----------|
|/     |GET     |Basic information about the Node.|
|/\<version number\>/blockchain|GET|Returns a list of Blocks in the Blockchain|
|/\<version number\>/blockchain/block|POST|Endpoint to announce a new Block during an epoch|
|/\<version number\>/network| GET| Returns a list of all known Nodes|
|/\<version number\>/network/node|POST|Endpoint to register a new Node|
|/\<version number\>/network/report|POST|Endpoint to report an offline Node|
|/\<version number\>/tx|POST|Endpoint to transmit a new transaction|

### /
This endpoint returns an object similar to this:
```
curl -X GET localhost:1234/
{"blockchain_address":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","node_name":"cypher-rust","node_version":"0.1.0","unix_time":1674812146914537}
```

|Field|Description|
------|-----------|
|blockchain_address|The Nodes Wallet Address, in the Ethereum format|
node_name|The Nodes software name. cypher-rust is the official implementation of the protocol.|
node_version|The nodes software version|
unix_time|The nodes current unix timestamp in milliseconds

### /\<version number\>/network
This endpoint returns an array similar to this:
```
curl -X GET localhost:1234/<version number>/network
[{"blockchain_address":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","ip_address":"192.168.178.22","registration_timestamp":1674812128203784}]
```
Each object in the array contains the following fields:

|Field|Description|
------|-----------|
|blockchain_address|The Nodes Wallet Address, in the Ethereum format|
|ip_address|The Nodes IPv4 address. IPv6 is not supported yet.
|registration_timestamp|The unix timestamp, of the moment, the node registered to the network

### /\<version number>/network/node
This endpoint returns an object similar to this:
```
curl -X POST -d "{\"ip_address\": \"123.123.123.123\", \"blockchain_address\": \"0x0000000000000000000000000000000000000000\", \"registration_timestamp\": $(date +%s%3N), \"version\": \"0.1.0\"}" localhost:1234/v0.1.0/network/node
{"status":true}
```

The endpoint expects the request body to be a `Node` struct. It returns an object containing a `status` value, of the type boolean. If the registration process succeeded, `status` will be `true`, else it will be `false`