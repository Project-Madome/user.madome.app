FROM ubuntu:focal

ARG BINARY_FILE

COPY $BINARY_FILE /madome-user
# COPY ./.env.docker /.env

EXPOSE 3112

ENTRYPOINT [ "/madome-user" ]
