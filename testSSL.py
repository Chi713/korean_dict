import ssl
import certifi
print(ssl.get_default_verify_paths())
print(certifi.where())
