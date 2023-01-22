FROM ubuntu:latest

RUN apt update && apt upgrade -y

COPY ./target/debug/cypher-rust ./

ENTRYPOINT [ "./cypher-rust" ]