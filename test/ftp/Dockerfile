FROM alpine:latest

RUN apk --no-cache add vsftpd

ENV FTP_USER=user
ENV FTP_PASS=pass
ENV PASV_ENABLE=YES
ENV PASV_MIN_PORT=21100
ENV PASV_MAX_PORT=21110
ENV PASV_ADDRESS=127.0.0.1
ENV ANON_ENABLE=NO
ENV NO_ANON_PASSWD=NO
ENV ANON_ROOT=/var/ftp

COPY vsftpd.sh /usr/sbin/

RUN chmod +x /usr/sbin/vsftpd.sh

EXPOSE 20 21

ENTRYPOINT ["/usr/sbin/vsftpd.sh"]