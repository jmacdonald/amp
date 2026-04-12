# syntax=docker/dockerfile:1.7
ARG BASE=alpine:3.20
FROM ${BASE} AS builder
WORKDIR /app
COPY --from=builder ["src/", "/app/src/"]
RUN echo "build $TARGETPLATFORM" && make all
RUN tool -v --help $FOO ${BAR} 3.3.6
ENV PORT=8080
EXPOSE 8080
CMD ["./bin/app", "--port", "${PORT}"]
