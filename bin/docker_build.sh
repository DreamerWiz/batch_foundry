#!/bin/zsh
proxy="127.0.0.1:7890"
docker buildx build --platform linux/amd64,linux/arm64 \
       -t registry.cn-shanghai.aliyuncs.com/brix/batch-foundry:latest \
       --push docker/foundry
