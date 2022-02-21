FROM ubuntu:focal

ARG BINARY_FILE

COPY $BINARY_FILE /madome-user

EXPOSE 3112

ENTRYPOINT [ "/madome-user" ]
