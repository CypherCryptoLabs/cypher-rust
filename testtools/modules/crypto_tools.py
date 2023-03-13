#!/bin/python3

# This script will generate the root_key (derivation path m/) from a seed phrase

import hashlib
from mnemonic import Mnemonic
import bip32utils
import secp256k1

def get_root_key():
    mnemon = Mnemonic('english')
    with open('seed_phrase.txt', 'r') as file:
        seed = mnemon.to_seed(file.read().strip())

    return bip32utils.BIP32Key.fromEntropy(seed)

def get_address():
    root_key = get_root_key()
    root_address = root_key.Address()
    # root_public_hex = root_key.PublicKey().hex()
    # root_private_wif = root_key.WalletImportFormat()
    # print('Root key:')
    # print(f'\tAddress: {root_address}')
    # print(f'\tPublic : {root_public_hex}')
    # print(f'\tPrivate: {root_private_wif}\n')

    return root_address

def sign_string(string):
    root_key = get_root_key().PrivateKey()
    private_key = secp256k1.PrivateKey(root_key, True)
    public_key = private_key.pubkey

    return private_key.ecdsa_sign(bytes(string, "utf-8"), raw=False, digest=hashlib.sha256)

def validate_signature(signature, message):
    root_key = get_root_key().PrivateKey()
    private_key = secp256k1.PrivateKey(root_key, True)
    public_key = private_key.pubkey

    return public_key.ecdsa_verify(bytes(message, "utf-8"), signature)