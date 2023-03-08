FROM minio/minio

RUN microdnf install nc

HEALTHCHECK --interval=3s --retries=20 --timeout=1s \
  CMD nc -z localhost 9000

EXPOSE 9000 9001

ENTRYPOINT ["/usr/bin/docker-entrypoint.sh"]
