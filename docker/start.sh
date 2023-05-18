docker-entrypoint.sh mysqld > /data/mariadb_log.txt & \
sleep 15 && \
/cypher/target/debug/cypher-rust > /data/log.txt 2>&1