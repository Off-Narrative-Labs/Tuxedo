# This is a multi-stage docker file. See https://docs.docker.com/build/building/multi-stage/
# for details about this pattern.



# For the build stage, we use an image provided by Parity
FROM docker.io/paritytech/ci-linux:production as builder
WORKDIR /wallet
#TODO The Workdir and Copy command is different here than in the node...
COPY . .
RUN cargo build --locked --release -p tuxedo-template-wallet


# For the second stage, we use a minimal Ubuntu image
FROM docker.io/library/ubuntu:20.04
LABEL description="Tuxedo Templet Wallet"

COPY --from=builder /wallet/target/release/tuxedo-template-wallet /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /node-dev node-dev && \
  mkdir -p /wallet-data /node-dev/.local/share && \
  chown -R node-dev:node-dev /wallet-data && \
  # Make the wallet data directory available outside the container.
  ln -s /wallet-data /node-dev/.local/share/tuxedo-template-wallet && \
  # unclutter and minimize the attack surface
  rm -rf /usr/bin /usr/sbin && \
  # check if executable works in this container
  /usr/local/bin/tuxedo-template-wallet --version

USER node-dev

EXPOSE 9944
VOLUME ["/wallet-data"]

ENTRYPOINT ["/usr/local/bin/tuxedo-template-wallet"]