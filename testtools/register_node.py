#!/bin/python3

# This test is currnetly supposed to fail, becuase the Node will try to look
# up the IP address in the dummy test and fail to talk to the supposed Node.
# TODO: start a http server and answer the Node with the data its looking for

from ast import Bytes
import modules.crypto_tools as crypto_tools
import modules.cypher as cypher

import secp256k1
import base58
import json
import requests
import time
import secrets

pk = secp256k1.PrivateKey

address = crypto_tools.get_address()
body = {
    "payload": {
        "ip_address": "127.0.0.1",
        "blockchain_address": address,
        "registration_timestamp": int(time.time() * 1000),
        "version": cypher.node_version,
    },
    "timestamp" : int(time.time() * 1000),
    "public_key" : crypto_tools.get_pub_key().serialize(False).hex()
}

body["signature"] = pk.ecdsa_serialize(pk, crypto_tools.sign_string(json.dumps(body))).hex()

# Try to register the local node
response = requests.post("http://localhost:" + str(cypher.node_port) + "/v" + cypher.node_version + "/network/node", json.dumps(body))

if response.status_code != 200:
    print("Node did not accept body!")
    exit(1)

if json.loads(response.text)["payload"]["status"] != False:
    print("Node reregistered an existing Node!")
    exit(1)

# Generate dummy data and try registering with it
random_bytes = secrets.token_bytes(32)
random_string = base58.b58encode(random_bytes).decode('utf-8')
random_string = random_string[:34]

body["payload"]["blockchain_address"] = random_string
body["payload"]["ip_address"] = "123.123.123.123"
body["signature"] = pk.ecdsa_serialize(pk, crypto_tools.sign_string(json.dumps(body))).hex()

response = requests.post("http://localhost:" + str(cypher.node_port) + "/v" + cypher.node_version + "/network/node", json.dumps(body))

if response.status_code != 200:
    print("Node did not accept body!")
    exit(1)

if json.loads(response.text)["payload"]["status"] == False:
    print("Node registered with dummy data failed!")
    exit(1)

print("Test was successful!")