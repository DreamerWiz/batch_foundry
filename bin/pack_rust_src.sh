#!/bin/bash
rm -rf rust-build/ && mkdir rust-build
content_list="src Cargo.toml"
for content in $content_list
do
    echo "cp $content rust-build/"
    cp -r $content rust-build/
done


rm -rf app/ && mkdir app
content_list="lib foundry.toml log4rs.yaml remappings.txt"
for content in $content_list
do
    echo "cp $content app/"
    cp -r $content app/ 
done

mkdir app/contracts && mkdir app/test
rm -rf tmp/log/* tmp/out/* tmp/run/*
    

gtar -zcvf app.tar.gz app/ && rm -rf app/ 
gtar -zcvf rust-build.tar.gz rust-build/ && rm -rf rust-build/ 
mv app.tar.gz docker/foundry/app.tar.gz
mv rust-build.tar.gz docker/foundry/rust-build.tar.gz