#!/bin/sh

adduser -G ftp -s /bin/sh -D $FTP_USER
echo "$FTP_USER:$FTP_PASS" | chpasswd

cat <<EOF >> /etc/vsftpd/vsftpd.conf
allow_writeable_chroot=YES
anon_root=$ANON_ROOT
anonymous_enable=$ANON_ENABLE
chroot_local_user=YES
ftpd_banner=Welcome to vsftpd
local_enable=YES
local_umask=022
log_ftp_protocol=YES
max_clients=10
max_per_ip=5
no_anon_password=$NO_ANON_PASSWD
passwd_chroot_enable=YES
pasv_address=$PASV_ADDRESS
pasv_enable=$PASV_ENABLE
pasv_max_port=$PASV_MAX_PORT
pasv_min_port=$PASV_MIN_PORT
seccomp_sandbox=NO
write_enable=YES
xferlog_std_format=YES
EOF

/usr/sbin/vsftpd /etc/vsftpd/vsftpd.conf
