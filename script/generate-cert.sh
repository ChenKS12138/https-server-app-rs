#!/usr/bin/env sh

rm -rf *.{pem,srl};

openssl genrsa -out ca.private.pem 4096\
 && openssl req -new -sha256 -key ca.private.pem -out ca.csr.pem -subj '/C=CN/ST=Jiangsu/L=Nanjing/O=Local Trusted CA Co./CN=local'\
 && openssl x509 -req -sha256 -in ca.csr.pem -out ca.cert.pem -signkey ca.private.pem -extfile ca.conf -CAcreateserial -extensions v3_ca -days 3650\
 && openssl genrsa -out server.private.pem 4096\
 && openssl req -new -sha256 -key server.private.pem -out server.csr.pem -subj '/C=CN/ST=Jiangsu/L=Nanjing/O=cattchen Co./CN=cattchen.local'\
 && openssl x509 -req -sha256 -in server.csr.pem -out server.cert.pem -CA ca.cert.pem -CAkey ca.private.pem -CAcreateserial -extfile ca.conf -extensions v3_cert -days 365\
 && cat server.cert.pem ca.cert.pem > server_bundle.cert.pem