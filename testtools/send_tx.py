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
        "amount": 1,
        "network_fee": 1,
        "sender_pub_key": crypto_tools.get_pub_key().serialize(False).hex(),
        "receiver_address": crypto_tools.get_address(),
        "timestamp": int(time.time() * 1000)
    },
    "timestamp" : int(time.time() * 1000),
    "public_key" : crypto_tools.get_pub_key().serialize(False).hex()
}

body["payload"]["signature"] = pk.ecdsa_serialize(pk, crypto_tools.sign_string(json.dumps(body["payload"], separators=(',', ':')))).hex()
body["signature"] = pk.ecdsa_serialize(pk, crypto_tools.sign_string(json.dumps(body, separators=(',', ':')))).hex()
print(json.dumps(body, separators=(',', ':')))

# Try to register the local node
response = requests.post("http://localhost:" + str(cypher.node_port) + "/v" + cypher.node_version + "/blockchain/tx", json.dumps(body, separators=(',', ':')))

if response.status_code != 200:
    print("Node did not accept body!")
    exit(1)

print(response.text)
if json.loads(response.text)["payload"]["status"] != False:
    print("Node reregistered an existing Node!")
    exit(1)

print("Tx sent")