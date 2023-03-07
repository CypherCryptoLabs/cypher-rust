#!/bin/python3

# This script will generate the root_key (derivation path m/) from a seed phrase

from mnemonic import Mnemonic
import bip32utils

mnemon = Mnemonic('english')
with open('seed_phrase.txt', 'r') as file:
    seed = mnemon.to_seed(file.read().strip())

root_key = bip32utils.BIP32Key.fromEntropy(seed)
root_address = root_key.Address()
root_public_hex = root_key.PublicKey().hex()
root_private_wif = root_key.WalletImportFormat()
print('Root key:')
print(f'\tAddress: {root_address}')
print(f'\tPublic : {root_public_hex}')
print(f'\tPrivate: {root_private_wif}\n')