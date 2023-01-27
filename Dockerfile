FROM ubuntu:latest
USER root

RUN apt update && apt upgrade -y

WORKDIR /
COPY ./target/debug/cypher-rust ./
RUN mkdir /data

CMD ["/bin/sh", "-c", "/cypher-rust > /data/log.txt 2>&1"]