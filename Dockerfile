FROM fedora:33
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/source_microservice /usr/local/bin/source_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["source_microservice"]
