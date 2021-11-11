FROM nginx:stable-alpine

RUN apk -U upgrade \
 && apk add --no-cache apache2-utils

RUN mkdir -p /srv \
    && chmod a+w /srv

EXPOSE 80
COPY nginx.conf /etc/nginx/conf.d/default.conf

CMD nginx -g "daemon off;"
