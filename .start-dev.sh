#!/bin/bash
cd /home/hjudgex/projects/swarmchat
export ANTHROPIC_BASE_URL="https://api.deepseek.com/anthropic"
export ANTHROPIC_API_KEY="sk-43a6b0f0fa854d30b9da8aaedfa25ebd"
export WSLENV="$WSLENV:ANTHROPIC_BASE_URL:ANTHROPIC_API_KEY"

exec claude --bare --dangerously-skip-permissions --model deepseek-v4-pro \
  -p "$(cat <<'PROMPT'
请阅读 SwarmChat-Project-Plan-For-AI-Agent.md.md 理解完整开发计划。

然后执行以下步骤：
1. 运行 `cd /home/hjudgex/projects/swarmchat && cargo check` 检查 scp-core 当前状态
2. 按 Phase 0 子任务列表从 P0-1 开始逐项推进
3. 确保每个模块编译通过
4. 每完成一个子任务输出状态报告

当前项目在 /home/hjudgex/projects/swarmchat
scp-core 在 /home/hjudgex/projects/swarmchat/scp-core
PROMPT
)" --print
