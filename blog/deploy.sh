#!/bin/bash
#
# Deploy to prod, woo!
#
set -e
SSH=$1

# Check connection
if ! ssh ${SSH} whoami; then
    echo "SSH connection failed"
    exit 1
fi

# Build the package for Linux x86-64 using musl.
rwf-cli package --target x86_64-unknown-linux-musl

ssh ${SSH} mkdir -p /root/blog
scp build.tar.gz ${SSH}:/root/blog/build.tar.gz
ssh ${SSH} tar -C /root/blog -xvf /root/blog/build.tar.gz

ssh ${SSH} tmux send-keys -t blog C-c
ssh ${SSH} tmux send-keys -t blog "./blog.bin"
ssh ${SSH} tmux send-keys -t blog Enter
