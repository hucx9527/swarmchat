#!/bin/bash
export ANTHROPIC_BASE_URL="https://api.deepseek.com/anthropic"
export ANTHROPIC_API_KEY="sk-43a6b0f0fa854d30b9da8aaedfa25ebd"
export WSLENV="$WSLENV:ANTHROPIC_BASE_URL:ANTHROPIC_API_KEY"
cd /home/hjudgex/projects/swarmchat
exec claude --dangerously-skip-permissions --model deepseek-v4-pro
