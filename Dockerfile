FROM ubuntu:16.04
MAINTAINER J. Dumont <j.dumont@coinamics.io>
RUN apt-get update && apt-get install -y libssl-dev pkg-config ca-certificates

EXPOSE 8080

ENV appname server-http

RUN mkdir /coinamics && mkdir /coinamics/${appname}
ADD target/release/server-http /coinamics/${appname}

RUN chmod 777 /coinamics/${appname}/server-http
CMD exec /coinamics/${appname}/server-http