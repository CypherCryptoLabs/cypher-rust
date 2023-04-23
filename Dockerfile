FROM mariadb:latest
USER root

RUN apt update && apt upgrade -y
RUN apt install python3 python3-pip openssl -y

WORKDIR /
RUN mkdir /cypher
COPY ./ ./cypher
RUN pip install -r /cypher/testtools/requirements.txt
RUN cp /cypher/docker/openssl.cnf /usr/lib/ssl/openssl.cnf
RUN mkdir /data

ENV MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=true
COPY docker/db_schema.sql /docker-entrypoint-initdb.d/
RUN chmod u+x /cypher/docker/start.sh

CMD ["/cypher/docker/start.sh"]