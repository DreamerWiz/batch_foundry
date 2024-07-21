FROM --platform=$TARGETPLATFORM rust as builder
ADD rust-build.tar.gz /root/
WORKDIR /root/rust-build
RUN cargo build --release
RUN cd foundry && cargo install --path ./crates/forge --profile release --force --locked

From --platform=$TARGETPLATFORM ubuntu
COPY --from=builder /root/rust-build/target/release/judger /bin/
COPY --from=builder /root/rust-build/foundry/target/release/forge /bin/
# COPY --from=builder /root/.foundry /root/.foundry
# RUN apt-get update
# RUN apt-get install curl -y \
#     && apt-get install git -y \
#     && apt-get install 
# RUN curl -L https://foundry.paradigm.xyz | bash && /root/.foundry/bin/foundryup && ln -s /root/.foundry/bin/forge /bin/forge
ADD app.tar.gz / 

RUN echo '#!/bin/bash \njudger server "$@"' > /usr/bin/server && chmod +x /usr/bin/server
RUN echo '#!/bin/bash \njudger client "$@"' > /usr/bin/client && chmod +x /usr/bin/client


WORKDIR /app

RUN /bin/judger init
CMD server