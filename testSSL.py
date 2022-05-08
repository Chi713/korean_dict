import ssl
import certifi

context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
context.load_verify_locations(certifi.where(),"./certs/cacert.pem")

print(ssl.get_default_verify_paths())
print(certifi.where())
