rm *.crt *.key *.csr *.srl

# 1. Generate CA
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
    -subj "/C=US/ST=State/L=City/O=YourCA/CN=localhost"

# 2. Generate server key and CSR using the config
openssl genrsa -out server.key 2048
openssl req -new -key server.key -out server.csr -config server.conf

# 3. Sign the certificate WITH the extensions
openssl x509 -req -days 365 -in server.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server.crt \
    -extfile server.conf -extensions v3_req

# Remove the CSR and SRL as they are not needed for testing self-signed certificates
rm server.csr ca.srl ca.key