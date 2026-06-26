# 058 - v0.3.0 Stable Gate Remediation V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 用户目标

在 `056` 完成顺序门禁、done gate 和 projection 收口之后，再补齐 `v0.3.0 stable` 仍然缺失的真正稳定门：

1. 验证必须可信；
2. 事件和 launch 必须原子；
3. ID 与路径必须安全；
4. provider 会话边界必须收紧；
5. Project Brain / Completion / Release 必须有正式入口；
6. closeout proof 必须来自 provider 真实状态；
7. workflow 责任必须拆清楚；
8. 最后用 CI / release gate / 真 E2E 给出稳定版证据。

一句话：

```text
把 v0.3.0 从“本地能跑的开发快照”补成“可发布的稳定链路”。
```

---

## 2. 背景

`057` 已经完成审计重基线。

当前结论是：

- `056A/056B/056C` 已经补上第一轮流程门禁；
- 旧审计口径不能继续原样使用；
- `v0.3.0 stable` 仍然被 8 类问题阻断。

本需求不再讨论旧审计是否正确，而是直接定义新的修复包。

---

## 3. 范围

### 3.1 必须做

1. Runtime 自执行验证与证据绑定
2. Event Store / Dispatcher 原子 claim 机制
3. Issue / Project / Release ID 与路径安全模型
4. Provider 运行时隔离与退出监督
5. Project Brain / Completion / Release 正式入口
6. Provider 查询驱动的 closeout attestation
7. Work / Delivery / Audit / Completion / Release saga 拆分
8. CI / release gate / 真 E2E 证据

### 3.2 涉及模块

- `crates/cli/**`
- `crates/task-loop/**`
- `crates/event-store/**`
- `crates/agent-dispatcher/**`
- `crates/spec/**`
- `crates/mcp/**`
- `crates/projection/**`
- `crates/release/**`
- `crates/task-artifacts/**`
- `apps/desktop/src-tauri/**`
- `apps/desktop/src/**`
- `docs/**`

---

## 4. 非目标

- 不回退 `056A/056B/056C` 已有实现。
- 不恢复旧 `input / execute / output` 架构。
- 不把 provider session 重新提升为业务 authority。
- 不在本需求里扩展新的聊天驱动多 agent 编排模式。
- 不先做页面美化优先于 stable gate。

---

## 5. 修复切片

### 058A - Trusted Validation Runtime

#### 目标

把当前“调用方上报验证结果”的模式，升级为“runtime 独立执行验证并签发证据”。

#### 必须实现

1. `validationCommands` 必须从 issue 合同读取，而不是从 completion request 直接信任。
2. runtime 自己执行命令，捕获：
   - `program`
   - `args`
   - `exitCode`
   - `stdout`
   - `stderr`
3. runtime 自己计算：
   - `changed files`
   - `base/head commit`
   - `working tree hash`
4. runtime 校验：
   - `allowed_paths`
   - `forbidden_paths`
5. evidence 必须绑定：
   - `issue_id`
   - `run_id`
   - `command_hash`
   - `changed_file_hash`
   - `validation_result_hash`
   - `generated_at`

#### 验收

- [ ] 调用方不能直接伪造 `exitCode=0` 让任务过关。
- [ ] changed files 不再只靠 completion request 上报。
- [ ] evidence 与当前 `run_id` 一一绑定。

### 058B - Atomic Event Claim And Run Allocation

#### 目标

让 `launch.requested -> launch.claimed -> session created` 形成真正安全的原子顺序。

#### 必须实现

1. launch claim 必须先落事件，再允许拉起 provider。
2. `run_id` / `session_id` / `event_id` 的分配不能再依赖“读全量事件后 +1”。
3. dispatcher 必须避免重复消费同一个 launch request。
4. idempotency key 检查不能只靠“先 load 再 find”。

#### 验收

- [ ] 同一 launch request 不会被两个 dispatcher 重复拉起。
- [ ] event / claim / run allocation 具备一致性。
- [ ] replay 后不会出现脏 claim 覆盖主链。

### 058C - Safe ID And Path Boundary

#### 目标

把 issue / project / release 的标识和路径边界收成可发布级模型。

#### 必须实现

1. `issueId`、`projectId`、`releaseId` 使用统一 typed validator。
2. 禁止：
   - 空值
   - `..`
   - `/`
   - `\\`
   - 逃逸性路径片段
3. artifact path 必须经过 containment 校验。
4. 所有任务写入路径必须能证明位于预期 root 内。

#### 验收

- [ ] 不能通过恶意 ID 逃出合法任务目录。
- [ ] 各类 artifacts 的相对路径统一归一化。

### 058D - Provider Runtime Isolation And Supervision

#### 目标

让外部 provider 从“能拉起来”升级到“边界受控、行为可证、失败可收”。

#### 必须实现

1. provider 会话要有明确隔离策略：
   - worktree / workspace 边界
   - 权限模式
   - timeout
   - cancel
   - exit proof
2. 去掉过于宽松的默认放行策略。
3. provider 的失败、超时、取消都要回写 authoritative facts。

#### 验收

- [ ] provider 会话失败不会只停留在日志层。
- [ ] provider 行为边界能被 runtime 和 projection 读取。
- [ ] worktree / workspace 约束可验证。

### 058E - Formal Project / Completion / Release Entry Points

#### 目标

把已经存在于 crate 内部的能力，提升为正式产品入口。

#### 必须实现

CLI 至少新增：

```text
agentflow project intake
agentflow project preview-goal
agentflow project confirm-goal
agentflow project confirm-plan
agentflow project materialize

agentflow completion inspect
agentflow completion decide

agentflow release prepare
agentflow release confirm
agentflow release publish
```

Desktop / Tauri 必须调用同一套 runtime API。

#### 验收

- [ ] UI 不再只是 projection / mock。
- [ ] Project Brain、Completion、Release 都有正式命令边界。

### 058F - Trusted Closeout Attestation

#### 目标

把 closeout proof 从“CLI 参数声明”升级成“provider 查询事实”。

#### 必须实现

1. GitHub / GitLab provider 必须提供：
   - merged query
   - issue closed query
2. closeout proof 由 runtime 调 provider 查询后写入。
3. `complete` 不再接受“只靠 `--merged` / `--issue-closed` 就过关”的模式。

#### 验收

- [ ] merged 和 issue closed 不能靠调用方自报。
- [ ] closeout proof 与 provider 查询结果一致。

### 058G - Workflow Saga Separation

#### 目标

把当前收在 closeout 路径里的多种职责，拆成清晰的 workflow saga。

#### 必须拆分

1. Work Loop
2. Delivery
3. Audit
4. Completion
5. Release

每段都要有：

- 输入事实
- 输出事实
- 可重放事件
- 对应 projection

#### 验收

- [ ] `complete` 不再承担过多横向职责。
- [ ] Delivery / Audit / Completion / Release 有明确边界。

### 058H - CI / Release Gate / Real E2E Evidence

#### 目标

给 `v0.3.0 stable` 补上真正可外部验收的发布证据。

#### 必须实现

1. 正式的 GitHub workflow / release gate
2. 至少一条真实 requirement 到 release 的 E2E
3. 外部可读的发布记录与 gate 结果

#### 验收

- [ ] 仓库存在正式 workflow
- [ ] 有真实运行证据，不只是本地 build 结果
- [ ] `v0.3.0 stable` 的发布结论可复查

---

## 6. 建议执行顺序

不要并行乱补，建议按这个顺序推进：

1. `058A` Trusted Validation Runtime
2. `058B` Atomic Event Claim And Run Allocation
3. `058C` Safe ID And Path Boundary
4. `058D` Provider Runtime Isolation And Supervision
5. `058E` Formal Project / Completion / Release Entry Points
6. `058F` Trusted Closeout Attestation
7. `058G` Workflow Saga Separation
8. `058H` CI / Release Gate / Real E2E Evidence

---

## 7. 依赖关系

### 强依赖

- `058F` 依赖 `058D`
- `058G` 依赖 `058A`、`058E`、`058F`
- `058H` 依赖 `058A` 到 `058G` 全部完成

### 弱依赖

- `058E` 最好在 `058A-058D` 之后做，避免先接产品入口，再返工底层 gate

---

## 8. GitHub Issues 拆分建议

建议直接拆成 8 条：

1. `AF-STABLE-001` Trusted Validation Runtime
2. `AF-STABLE-002` Atomic Event Claim And Run Allocation
3. `AF-STABLE-003` Safe ID And Path Boundary
4. `AF-STABLE-004` Provider Runtime Isolation And Supervision
5. `AF-STABLE-005` Formal Project / Completion / Release Entry Points
6. `AF-STABLE-006` Trusted Closeout Attestation
7. `AF-STABLE-007` Workflow Saga Separation
8. `AF-STABLE-008` CI / Release Gate / Real E2E Evidence

---

## 9. v0.3.0 Stable 的最终验收标准

只有同时满足下面条件，才允许把版本结论从：

```text
NO-GO / development snapshot
```

升级为：

```text
GO / v0.3.0 Stable
```

### 必须全部成立

- [ ] Project Brain / Completion / Release 有正式 CLI / Desktop 入口
- [ ] Validation 由 runtime 自执行，不接受调用方伪造结果
- [ ] launch claim / event / run allocation 原子一致
- [ ] issue/project/release ID 与 artifact path 安全
- [ ] provider runtime 有隔离、监督、超时、取消和退出证明
- [ ] merge / issue closed 由 provider 查询写入 closeout proof
- [ ] Work / Delivery / Audit / Completion / Release saga 分层明确
- [ ] 仓库具备正式 CI / release gate / 真实 E2E 证据

---

## 10. 验证命令

至少覆盖：

```text
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

以及：

- runtime validation integration tests
- dispatcher / event-store concurrency tests
- provider closeout query tests
- one real E2E release gate proof
