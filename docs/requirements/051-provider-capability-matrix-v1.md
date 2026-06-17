# 051 - Provider Capability Matrix V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

随着 provider 扩展，必须明确：

- 哪个 provider 支持什么能力；
- 哪个 role 可以绑定哪个 provider；
- dispatcher 如何选择 provider。

---

## 2. 用户目标

用户和系统都能解释：

- 为什么这条任务选了这个 provider；
- 这个 provider 到底支持哪些能力；
- 哪些能力缺失时系统会如何降级。

---

## 3. 范围

### 3.1 必须做

1. 定义 provider capability schema。
2. 定义 role / skill / provider compatibility。
3. 定义 dispatcher selection rules。
4. 定义 capability 缺失时的处理方式。

### 3.2 涉及模块

- `crates/mcp/**`
- `crates/agent-dispatcher/**`
- `crates/spec/**`
- `crates/projection/**`

---

## 4. 关键设计要求

### 4.1 能力矩阵是显式事实

- 不是靠代码里散落判断。

### 4.2 选择规则可解释

- dispatcher 必须能解释为什么选这个 provider。

### 4.3 不做自动抢占

- 本阶段只做选择规则，不做 provider 自动抢任务。

---

## 5. 非目标

- 不做 provider 自动竞价或自动抢占。
- 不把 capability matrix 做成业务 authority。

---

## 6. 依赖

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)
- [049-codex-provider-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/049-codex-provider-hardening-v1.md)

---

## 7. 验收标准

- [ ] provider 能力边界清楚。
- [ ] dispatcher 选择规则可解释。
- [ ] role / skill / provider 兼容关系清晰。

---

## 8. 验证命令

- `cargo test --workspace`
- `git diff --check`
