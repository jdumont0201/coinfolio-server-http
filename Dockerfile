FROM ubuntu:16.04
MAINTAINER J. Dumont <j.dumont@coinamics.io>
RUN apt-get update && apt-get install -y libssl-dev pkg-config ca-certificates

EXPOSE 8080

ENV appname server-http

RUN mkdir /coinfolio && mkdir /coinfolio/${appname}
ADD target/release/server-http /coinfolio/${appname}

RUN chmod 777 /coinfolio/${appname}/server-http
CMD exec /coinfolio/${appname}/server-http