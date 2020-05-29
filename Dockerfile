FROM ubuntu:focal as builder

ADD . /src
WORKDIR /src

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
        pkg-config \
        clang \
        curl \
        libssl-dev \
        libavcodec-dev \
        libavdevice-dev \
        libavfilter-dev \
        libavformat-dev \
        libavresample-dev \
        libpostproc-dev \
        libswresample-dev \
        ffmpeg \
        && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="/root/.cargo/bin:${PATH}" && \
    cargo build --verbose --release && \
    cargo install --path .

FROM ubuntu:focal
COPY --from=builder /root/.cargo/bin/media_splitter_worker /usr/bin

RUN apt update && \
    apt install -y \
        libssl1.1 \
        ca-certificates \
        ffmpeg

ENV AMQP_QUEUE job_media_splitter
CMD media_splitter_worker
