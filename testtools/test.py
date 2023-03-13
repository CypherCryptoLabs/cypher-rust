import modules.crypto_tools as crypto_tools

signature = crypto_tools.sign_string("test")
validity = crypto_tools.validate_signature(signature, "test")

if validity == True:
    print("works")