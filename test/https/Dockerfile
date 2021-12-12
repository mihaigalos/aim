FROM nginx:stable-alpine

RUN apk -U upgrade \
    && apk add --no-cache \
        apache2-utils \
        openssl

RUN mkdir -p /srv \
    && chmod a+w /srv \
    && printf "user:$(openssl passwd -crypt pass)\n" > /etc/nginx/.htpasswd;

EXPOSE 80
COPY nginx.conf /etc/nginx/conf.d/default.conf

CMD nginx -g "daemon off;"
