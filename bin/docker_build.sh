#!/bin/zsh
proxy="127.0.0.1:7890"

docker login --username=观维科技 registry.cn-shanghai.aliyuncs.com -p 6YzYtxFeiK9GvSKg

docker buildx create --use --driver-opt env.http_proxy=172.17.0.1:7890 --driver-opt env.https_proxy=172.17.0.1:7890 --driver-opt env.no_proxy='localhost,127.0.0.1'

docker buildx build --platform linux/amd64,linux/arm64 \
       -t registry.cn-shanghai.aliyuncs.com/brix/batch-foundry:latest \
       --push docker/foundry

# docker build -t registry.cn-shanghai.aliyuncs.com/brix/batch-foundry:latest --build-arg http_proxy=http://host.docker.internal:7890 --build-arg https_proxy=http://host.docker.internal:7890 docker/foundry