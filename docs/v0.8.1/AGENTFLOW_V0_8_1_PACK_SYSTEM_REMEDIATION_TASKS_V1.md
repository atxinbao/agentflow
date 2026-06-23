# AgentFlow v0.8.1 Pack System Remediation Tasks V1

日期：2026-06-23
执行者：Codex

## Goal

`v0.8.1` 目标是修复 `v0.8.0` Pack System 发布审计后暴露的问题。

主线是：

```text
File-backed Pack Registry
-> Registry-driven Command Resolver
-> Pack-specific Projection
-> Capability-aware Command Availability
-> Invalid Command Rejection Boundary
-> Negative Fixtures
-> Release Certification
```

本版本不是新架构版本。它是 `v0.8.0` Pack System 的真实化修复版本。

## Product Principle

Pack System 的核心不是把两个行业壳写进代码里，而是让行业定义成为可读取、可验证、可迁移的项目文件。

正确边界是：

```text
Pack 文件定义行业现场。
Runtime Registry 读取 Pack。
Command Resolver 解析合法命令。
Projection Surface 按 Pack 定义展示。
Capability Registry 决定命令是否可用。
Release Gate 用失败夹具证明边界有效。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V081-001` | Pack Registry File-backed Source of Truth | P0 | none | done |
| `V081-002` | Runtime Pack Command Resolver Uses Registry | P0 | V081-001 | done |
| `V081-003` | Projection Loads Pack-specific Definitions | P0 | V081-001 | done |
| `V081-004` | Capability Status Uses Capability Registry / Provider Smoke | P0 | V081-002 | done |
| `V081-005` | Pack Submit Rejection Boundary | P0 | V081-002 | done |
| `V081-006` | Release Summary Audit Sidecar Wording | P1 | none | done |
| `V081-007` | Pack Release Gate Negative Fixtures | P0 | V081-001, V081-002, V081-003, V081-004, V081-005 | done |
| `V081-008` | v0.8.1 Release Audit Certification | P0 | V081-001, V081-002, V081-003, V081-004, V081-005, V081-006, V081-007, V081-009 | done |
| `V081-009` | Project Structural Information Principle | P1 | none | done |

## V081-001 Pack Registry File-backed Source of Truth

### Scope

把 Software Dev Pack / UI Design Pack 落成可读取的 Pack 文件或 fixture。

必须处理：

- Pack Registry 读取真实文件或 test fixture；
- Software Dev Pack 和 UI Design Pack 不再只依赖 built-in baseline；
- release gate 能证明 registry 数据来自文件；
- built-in fallback 如果仍保留，必须显式标记为 fallback，不得伪装成 Pack source of truth。

### Acceptance

- 存在可读取的 Pack 文件或 fixture；
- registry loader 可以列出 Software Dev Pack / UI Design Pack；
- release gate 使用 file-backed registry 输入；
- 测试或 fixture 证明缺文件时不能悄悄回退成 built-in success。

## V081-002 Runtime Pack Command Resolver Uses Registry

### Scope

Command Surface 必须从 registry、domain、surface、connector 定义解析命令，不再只认 hardcoded built-ins。

必须处理：

- resolver 以 Pack registry 为入口；
- resolver 读取 domain action semantics；
- resolver 读取 surface command mapping；
- resolver 读取 connector / capability requirements；
- custom Pack command 不能被 Software Dev built-in 逻辑吞掉。

### Acceptance

- 自定义 Pack command 可以通过 registry 解析；
- 缺少 action、surface command mapping 或 connector requirement 时返回明确错误；
- tests 覆盖 built-in Pack 和 custom fixture Pack；
- resolver 不直接写 authority。

## V081-003 Projection Loads Pack-specific Definitions

### Scope

Projection 必须加载 Pack-specific definitions。

自定义 Pack 不能默认回退成 Software Dev 定义。

必须处理：

- read model 使用当前 Pack 的 object / view model / surface 定义；
- missing definition 显示 invalid / deferred；
- UI Design Pack 不能被渲染成 Software Dev Project Home；
- Projection 错误状态保持只读，不发起修复命令。

### Acceptance

- Projection 能加载 Software Dev Pack 和 UI Design Pack 的不同定义；
- custom Pack 缺 read model definition 时显示 invalid / deferred；
- tests 证明不会 fallback 成 Software Dev；
- Projection 仍然只读。

## V081-004 Capability Status Uses Capability Registry / Provider Smoke

### Scope

Pack command availability 必须受真实 capability / provider smoke 状态影响。

必须处理：

- capability registry 暴露 capability enabled / disabled / degraded 状态；
- provider smoke 结果能影响 connector-backed command availability；
- disabled capability 下 command 不能显示为可执行；
- command availability 需要说明不可用原因。

### Acceptance

- disabled provider / capability fixture 会禁用对应 Pack command；
- degraded 状态不会被误报成 ready；
- release gate 覆盖 disabled capability；
- command availability 可追溯到 capability registry 或 provider smoke 输入。

## V081-005 Pack Submit Rejection Boundary

### Scope

invalid Pack command 不应进入提交路径。

只允许产生 rejected validation report。

必须处理：

- invalid command 不进入 Runtime submit；
- invalid command 不进入 Arbitration；
- invalid command 不写 accepted proposal；
- invalid command 产生 rejected validation report；
- report 包含失败阶段、原因、关联 Pack、关联 command。

### Acceptance

- invalid Pack command 被拒绝在 submit 前；
- tests 证明不会产生 accepted action proposal；
- rejected validation report 可被 release gate 读取；
- failure reason 明确区分 schema、read model、connector、capability、surface mapping。

## V081-006 Release Summary Audit Sidecar Wording

### Scope

修正 release summary 中 `auditStatus: failed` 这类容易混淆的表达。

Audit 是 sidecar，不等于 release gate 主结论。

必须处理：

- 将 `auditStatus` 改成 `sidecarAuditStatus`，或放入独立 audit sidecar 区块；
- release gate conclusion 与 sidecar audit result 分开表达；
- failed sidecar audit 不能被误读成 release gate failed，除非 release policy 明确绑定；
- 文档说明 Audit 不回到 Software Dev Pack 主业务链路。

### Acceptance

- release summary wording 不再混淆 release gate 与 audit sidecar；
- tests / fixtures 覆盖 sidecar audit failed 但 release gate 主链可独立表达的场景；
- docs 继续保持 Audit independent flow。

## V081-007 Pack Release Gate Negative Fixtures

### Scope

增加 Pack release gate negative fixtures。

必须覆盖：

- invalid Pack；
- missing read model；
- missing connector；
- disabled capability；
- invalid command submit；
- unexpected Software Dev fallback。

### Acceptance

- 每个 negative fixture 都会在正确阶段失败；
- failure report 包含原因和 stage；
- release gate 不能只用 happy path 证明 readiness；
- negative fixture 失败不会写 authority。

## V081-008 v0.8.1 Release Audit Certification

### Scope

证明 `v0.8.1` 修复后 Pack System 才是真正 file-driven baseline。

必须输出：

- release certification artifact；
- Pack registry source proof；
- command resolver proof；
- projection pack-specific proof；
- capability/provider smoke proof；
- invalid submit rejection proof；
- negative fixtures report；
- audit sidecar wording proof；
- remaining risk / deferred list。

### Acceptance

- `v0.8.1` release gate 覆盖 V081-001 到 V081-007；
- certification 文档明确当前版本是否 clean remediation release；
- 所有未完成能力必须进入后续版本，不得伪装完成；
- 不创建 Audit Issue，除非人类明确发起独立审计。

## V081-009 Project Structural Information Principle

### Scope

补入项目结构化信息原则，吸收 arXiv:2601.03220 对 agent 系统结构信息、任务结构和复杂度表达的启发。

参考来源：

- https://arxiv.org/abs/2601.03220

本 issue 不要求形式化实现论文中的全部指标。它只要求把原则落到 AgentFlow 可执行的项目信息结构里。

必须处理：

- Requirement / Spec / Issue / Run / Evidence / Projection 之间的结构关系；
- Pack 对对象、动作、状态、证据的定义边界；
- Projection 不能丢失结构，只做 summary；
- Spec Loop 和 Work Loop 必须保留可追溯的结构化信息；
- release gate 至少能检查结构信息是否断链。

### Acceptance

- 文档或 schema 明确 Project Structural Information Principle；
- 至少一个 fixture 证明 requirement -> spec -> issue -> run -> evidence -> projection 的结构可追溯；
- Pack-specific Projection 不把结构压扁成不可验证文本；
- 不引入论文指标的伪实现。

## Execution Order

建议执行顺序：

```text
V081-001
-> V081-002 + V081-003
-> V081-004 + V081-005
-> V081-006
-> V081-007
-> V081-009
-> V081-008
```

`V081-008` 必须最后执行。

## Release Certification

`V081-008` 的收口标准是 release gate 能同时证明：

- Pack registry source 来自 file-backed / fixture-backed 输入；
- command resolver 不再依赖 built-in Software Dev fallback；
- Projection 读取 Pack-specific definitions；
- capability / provider smoke 会影响 command availability；
- invalid submit 在 Runtime authority 写入前被拒绝；
- negative fixtures 覆盖失败阶段和原因；
- Audit sidecar wording 不覆盖 release gate 主结论；
- 未完成能力转入后续版本，不伪装为完成。
