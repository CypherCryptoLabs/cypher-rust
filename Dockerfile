FROM ubuntu:latest
USER root

RUN apt update && apt upgrade -y
RUN apt install python3 python3-pip openssl -y

WORKDIR /
RUN mkdir /cypher
COPY ./ ./cypher
RUN pip install -r /cypher/testtools/requirements.txt
RUN cp /cypher/docker/openssl.cnf /usr/lib/ssl/openssl.cnf
RUN mkdir /data

CMD ["/bin/sh", "-c", "/cypher/target/debug/cypher-rust > /data/log.txt 2>&1"]