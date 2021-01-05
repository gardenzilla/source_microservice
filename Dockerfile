FROM debian:buster-slim
WORKDIR /usr/local/bin
COPY ./target/release/source_microservice /usr/local/bin/source_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["source_microservice"]