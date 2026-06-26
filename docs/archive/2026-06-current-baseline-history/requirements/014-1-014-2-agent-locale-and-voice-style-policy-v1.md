# 014.1 + 014.2 - Agent Locale and Voice Style Policy V1

创建日期：2026-06-05
执行者：Codex
状态：已开发
版本：final

---

## 用户目标

在进入 `015 - Input SPEC Authoring & Approval V1` 之前，需要先补两条 Agent 基础规则：

```text
014.1 - Agent Locale Policy V1
014.2 - Agent Voice Style Policy V1
```

这两个需求解决的是同一个产品问题：

> Agent 不能一会儿中文、一会儿英文；也不能写出 AI 腔、官话、营销腔。
> AgentFlow 必须先给 Agent 定好“用什么语言”和“用什么风格”。

---

## 一句话定义

> **014.1 + 014.2 负责给 AgentFlow 的 Agent 定默认语言和默认表达风格：AgentFlow managed manuals 统一英文；Agent 面向用户的自然语言输出和新写代码注释跟随用户 OS locale；所有输出使用 plain-work-style，要求直接、清楚、少废话、有证据、有下一步。**

---

# 1. 总体规则

最终定案：

```text
manualLanguage = en
agentLocale = 用户系统 OS / App locale
voiceStyle = plain-work-style
```

解释：

```text
AgentFlow 工作手册统一英文
Agent 面向用户输出跟随 agentLocale
Agent 新写代码注释跟随 agentLocale
Agent 输出风格统一为 plain-work-style
```

大白话：

> 手册用英文，保证规则稳定。
> Agent 跟用户说话时，用用户系统语言。
> Agent 写新代码注释时，也用用户系统语言。
> 不管用什么语言，都要说人话。

---

# 2. 014.1 - Agent Locale Policy V1

## 2.1 目标

AgentFlow 在项目环境准备阶段检测用户 OS / App locale，并记录为：

```text
agentLocale
```

Agent 后续所有用户面自然语言输出必须跟随：

```text
agentLocale
```

不要只支持：

```text
zh-CN / en
```

而是支持任意 BCP 47 locale 字符串，例如：

```text
zh-CN
zh-TW
en-US
en-GB
ja-JP
ko-KR
fr-FR
de-DE
es-ES
pt-BR
```

---

## 2.2 Locale 来源优先级

语言来源优先级：

```text
1. OS locale
2. App locale
3. Browser Preview navigator.language / Intl locale
4. Existing workspace locale
5. fallback: en-US
```

正常情况下：

```text
agentLocale = OS / App locale
```

只有无法检测 locale 时：

```text
agentLocale = en-US
fallback = true
warnings 记录原因
```

---

## 2.3 需要保存完整 locale

不要只保存：

```text
zh
en
```

必须保存完整 BCP 47 locale：

```text
zh-CN
en-US
ja-JP
pt-BR
```

原因：

```text
zh-CN / zh-TW 不一样
en-US / en-GB 不一样
pt-PT / pt-BR 不一样
```

---

## 2.4 manualLanguage 固定为英文

以下 AgentFlow managed manuals 统一英文，不随 locale 翻译：

```text
AGENTS.md 正文
Agentflow.md 正文
skills/SKILL.md 正文
SPEC.md 正文
TDD.md 正文
RELEASE.md 正文
AUDIT.md 正文
Agent output rules 说明
Browser Preview mock 文案
AgentFlow managed status / policy 文案
```

原因：

```text
1. 工作手册是给 Agent / developer / automation 读的。
2. 英文更稳定，方便和 Codex / Claude / Zed / Warp 等生态兼容。
3. 不需要维护多语言模板。
4. 避免 OS locale 切换导致整套手册频繁重写和 hash 波动。
```

---

## 2.5 agentLocale 覆盖哪些内容

这些内容必须跟随 `agentLocale`：

```text
Agent 会话回复
需求澄清问题
Requirement Intake Result 的 summary / explanation / questions
SPEC Draft Preview 的自然语言正文
Approved SPEC 的 product / tech / tasks 自然语言内容
Issue title / summary / acceptance criteria
TDD plan 正文
Release note 正文
Audit report 正文
给用户看的错误说明 / 阻断原因
Agent 新增的代码注释
Agent 新增的 doc comment
Agent 新增的 inline comment
Agent 新增的测试说明性注释
Agent 新增的 TODO / FIXME 说明
```

示例：

```text
OS locale = zh-CN
→ Agent 回复中文简体
→ SPEC 草案中文简体
→ Issue 描述中文简体
→ 代码注释中文简体

OS locale = ja-JP
→ Agent 回复日文
→ SPEC 草案日文
→ Issue 描述日文
→ 代码注释日文
```

---

## 2.6 不跟随 agentLocale 的内容

以下内容保持原样，不翻译：

```text
文件名
目录名
JSON key
enum value
Rust / TypeScript / Python / Java 等代码标识符
函数名
变量名
类型名
crate name
package name
Tauri command
CLI command
shell command
API name
第三方库名
协议名
错误码
配置 key
路径
```

示例：

```json
{
  "agentLocale": "zh-CN",
  "status": "ready",
  "currentStage": "delivery-ready"
}
```

这里：

```text
agentLocale
status
currentStage
delivery-ready
```

都不翻译。

---

## 2.7 代码注释语言规则

Agent 新写的代码注释必须跟随：

```text
agentLocale
```

中文环境：

```rust
// 检查 Agent 工作手册是否完整。
fn validate_agent_manual() {}
```

英文环境：

```rust
// Validate that the Agent working manual is complete.
fn validate_agent_manual() {}
```

日文环境：

```rust
// Agent 作業マニュアルが完全であることを検証する。
fn validate_agent_manual() {}
```

---

## 2.8 不主动翻译已有注释

Agent 不应该因为 locale 变化就批量翻译已有代码注释。

规则：

```text
Do not mass-translate existing comments.
```

原因：

```text
这会制造无关 diff
也可能破坏项目原来的代码风格
```

只有在必须修改某段代码时，才可以顺带调整相关注释。

规则：

```text
When editing an existing comment as part of a necessary code change, the updated comment should follow agentLocale.
Do not modify comments solely for translation.
```

---

## 2.9 协议 / 标准 / API 注释可保留原文术语

有些注释需要保留英文术语，例如：

```text
SAFETY
HTTP header
OAuth scope
OpenAPI field
Kubernetes resource
Rust unsafe safety contract
TypeScript public API doc
第三方库约定
```

如果翻译会降低准确性，可以保留术语，解释部分跟随 `agentLocale`。

中文环境示例：

```rust
// SAFETY: 这里必须保证 ptr 非空，并且生命周期不超过 buffer。
unsafe fn read_ptr(ptr: *const u8) {}
```

---

# 3. 014.2 - Agent Voice Style Policy V1

## 3.1 目标

给 AgentFlow 的 Agent 增加默认语言风格规则：

```text
plain-work-style
```

这个规则不是“去 AI 味后处理”。

它是：

```text
Agent 默认说话规则
```

大白话：

> Agent 不是先写一版 AI 腔，再改成大白话。
> Agent 一开始就必须按 plain-work-style 输出。

---

## 3.2 plain-work-style 核心

无论 `agentLocale` 是什么语言，Agent 都必须遵守：

```text
先说结论
讲人话
少废话
少套话
少营销腔
不装高级
不假装确定
给明确下一步
```

---

## 3.3 默认输出结构

除非用户明确要求别的格式，Agent 默认按这个顺序输出：

```text
1. 结论
2. 依据
3. 问题
4. 下一步动作
```

如果用户要的是 Codex / Agent 指令，直接给可复制指令，不要先讲长背景。

---

## 3.4 必须遵守的表达规则

Agent 输出必须遵守：

```text
先说结论
每段只讲一件事
多用短句
多用普通词
不要堆概念
不要写空话
不要写官话
不要写营销腔
不要写公众号腔
不要装作很确定
没有证据就说没看到证据
能给动作就给动作
```

---

## 3.5 禁止的 AI 腔

禁止这类句式：

```text
在当今快速发展的时代……
随着技术的不断进步……
这不仅是 A，更是 B……
综上所述……
值得注意的是……
不难看出……
可以说……
毫无疑问……
这将极大提升……
为用户带来更好的体验……
打造完整生态闭环……
赋能业务增长……
```

少用或禁用这类空泛词：

```text
赋能
打造
沉淀
闭环
抓手
全链路
多维度
生态
范式
体系化
深度
全面
革新
领先
极大提升
```

注意：

```text
如果某些词是项目里的正式术语，可以使用。
但必须先解释它具体指什么。
```

---

## 3.6 技术解释规则

Agent 解释技术时必须：

```text
先讲人话
再讲术语
最后讲怎么做
```

坏例子：

```text
该模块通过多维状态协同，实现任务生命周期闭环管理。
```

好例子：

```text
这个模块只做一件事：记录任务现在走到哪一步，下一步该谁处理。
```

---

## 3.7 项目分析规则

Agent 做项目分析时，不能写泛泛的：

```text
优势
挑战
展望
未来可期
生态价值
```

必须回答：

```text
当前是什么状态
问题在哪里
哪些马上要补
哪些可以后面做
哪些现在不要做
怎么验证
```

---

## 3.8 Codex 指令规则

Agent 写 Codex 指令时，必须直接给可执行指令。

默认结构：

```text
背景
目标
范围
步骤
禁止事项
验证方式
输出要求
```

不要写：

```text
请你全面分析并基于最佳实践进行优化
```

要写：

```text
你只做三件事：
1. 找到和本需求相关的文件。
2. 对照需求逐条检查是否实现。
3. 没实现的地方只做最小改动，不要顺手重构。
```

---

## 3.9 不确定时的表达

应该说：

```text
我没看到证据说明这个已经完成。
从目前信息看，更像是 A，不像是 B。
这里需要再看代码才能确认。
这个判断有一个前提：xxx。
```

不要说：

```text
显然……
必然……
完全可以证明……
一定是……
毫无疑问……
```

---

## 3.10 代码注释风格

Agent 新写代码注释时必须同时满足：

```text
语言跟随 agentLocale
风格遵守 plain-work-style
```

代码注释必须：

```text
短
准
说明为什么
不要重复代码本身
不要写废话
不要营销腔
不要为了显得专业而堆术语
```

坏例子：

```rust
// This function comprehensively facilitates the holistic validation of the workflow ecosystem.
```

好例子：

```rust
// Validate the workflow state before allowing the next action.
```

---

# 4. 状态文件设计

## 4.1 locale.json

新增或扩展：

```text
.agentflow/define/agent/state/locale.json
```

示例：

```json
{
  "version": "agent-locale.v1",
  "agentLocale": "zh-CN",
  "rawOsLocale": "zh-CN",
  "manualLanguage": "en",
  "source": "os",
  "checkedAt": 1780600000,
  "fallback": false,
  "warnings": []
}
```

字段说明：

```text
agentLocale
= Agent 用户面输出语言，跟随 OS / App locale

manualLanguage
= AgentFlow managed manuals 的语言，固定 en

rawOsLocale
= 检测到的原始 OS / App locale

source
= os / app / browser-preview / existing-workspace / fallback

fallback
= 是否因无法检测 locale 而回退到 en-US
```

---

## 4.2 style.json

新增：

```text
.agentflow/define/agent/state/style.json
```

示例：

```json
{
  "version": "agent-style.v1",
  "styleId": "plain-work-style",
  "manualLanguage": "en",
  "appliesToAgentLocale": true,
  "appliesToCodeComments": true,
  "checkedAt": 1780600000,
  "warnings": []
}
```

字段说明：

```text
styleId
= 当前默认 Agent voice style

appliesToAgentLocale
= 该风格适用于所有 agentLocale

appliesToCodeComments
= 该风格也约束 Agent 新写代码注释
```

---

## 4.3 workspace-manifest.json

增加 locale / style 信息：

```json
{
  "locale": {
    "agentLocale": "zh-CN",
    "manualLanguage": "en",
    "rawOsLocale": "zh-CN",
    "source": "os",
    "checkedAt": 1780600000,
    "fallback": false,
    "warnings": []
  },
  "style": {
    "styleId": "plain-work-style",
    "manualLanguage": "en",
    "appliesToAgentLocale": true,
    "appliesToCodeComments": true
  }
}
```

---

## 4.4 skills-lock.json

增加 metadata：

```json
{
  "version": "agentflow-skills-lock.v1",
  "manualLanguage": "en",
  "agentLocale": "zh-CN",
  "stylePolicy": {
    "styleId": "plain-work-style",
    "version": "v1",
    "path": ".agentflow/define/agent/skills/plain-work-style/SKILL.md",
    "appliesToCodeComments": true
  }
}
```

注意：

```text
agentLocale 变化不应该导致 skills/SKILL.md hash 变化
因为 SKILL.md 正文统一英文
```

Locale 变化时只更新：

```text
locale.json
workspace-manifest.json
skills-lock.json metadata
state status
```

不重写：

```text
AGENTS.md
Agentflow.md
skills/SKILL.md
SPEC.md
TDD.md
RELEASE.md
AUDIT.md
```

除非这些手册本身版本变化或 hash mismatch。

---

# 5. 新增 skill

新增：

```text
.agentflow/define/agent/skills/plain-work-style/SKILL.md
```

该 skill 正文统一英文。

必须包含：

```text
Purpose
Default output structure
Plain language rules
Forbidden tone
Technical explanation rules
Project analysis rules
Codex instruction rules
Uncertainty rules
Code comment rules
Output self-check
```

当前 Agent skills 从 7 个变成 8 个，具体以项目当前数量为准，但必须新增：

```text
plain-work-style
```

注意：

```text
plain-work-style 是默认 voice policy，不是可选改写工具。
```

---

# 6. AGENTS.md 更新

AGENTS.md 正文保持英文。

增加：

```md
## Locale Policy

- AgentFlow managed manuals are written in English.
- The Agent MUST use the detected `agentLocale` for all user-facing natural-language output.
- The Agent MUST use `agentLocale` for newly authored code comments and doc comments.
- Do not mass-translate existing comments.
- Keep filenames, paths, code identifiers, JSON keys, enum values, command names, and API names unchanged.

## Voice Style Policy

- Agent user-facing output MUST follow the plain-work-style policy.
- Start with the conclusion.
- Use plain language.
- Avoid filler, marketing tone, and vague claims.
- Be specific about evidence, gaps, risks, and next actions.
- If evidence is missing, say that evidence is missing.
- Newly authored code comments and doc comments MUST follow `agentLocale` and plain-work-style.
- Do not mass-translate existing code comments.
```

---

# 7. Agentflow.md 更新

Agentflow.md 正文保持英文。

新增：

```md
## Locale Policy

Manual language is always English.

The Agent's user-facing natural-language output MUST follow `agentLocale`.

This includes:
- conversation replies
- clarification questions
- Requirement Intake Result explanations
- SPEC Draft Preview prose
- Issue titles and summaries
- acceptance criteria prose
- TDD plans
- release notes
- audit reports
- user-facing blocker explanations
- newly authored code comments
- newly authored doc comments

Do not translate:
- filenames
- paths
- code identifiers
- JSON keys
- enum values
- command names
- crate/package names
- API names

Do not mass-translate existing code comments. When editing a comment as part of a necessary code change, the updated comment should follow `agentLocale`.
```

新增：

```md
## Voice Style Policy

AgentFlow uses `plain-work-style` as the default Agent voice.

This policy applies to:
- conversation replies
- requirement clarification
- Requirement Intake Result explanations
- SPEC Draft Preview prose
- Issue summaries
- acceptance criteria prose
- TDD plans
- release notes
- audit reports
- user-facing blocker explanations
- newly authored code comments
- newly authored doc comments

Rules:
- Start with the conclusion.
- Use plain, direct language.
- Avoid filler, hype, marketing tone, and abstract buzzwords.
- Prefer concrete next actions.
- Do not pretend to be certain without evidence.
- Keep code identifiers, file names, JSON keys, commands, and paths unchanged.
- Do not mass-translate existing code comments.
```

---

# 8. TDD.md / Build Agent 规则更新

虽然 Build Agent 当前未授权，但 TDD.md 必须提前写入代码注释规则：

```md
## Code Comment Language and Style

When Build Agent becomes authorized, any newly authored code comment, test comment, or doc comment MUST follow `agentLocale` and `plain-work-style`.

Do not rewrite existing comments only to change their language.
```

---

# 9. validate / repair 要求

## 9.1 validate 必须能发现

```text
locale.json 缺失
locale.json manualLanguage != en
locale.json agentLocale 缺失
style.json 缺失
style.json styleId != plain-work-style
style.json appliesToCodeComments != true
plain-work-style/SKILL.md 缺失
plain-work-style hash mismatch
skills-lock.json 缺少 stylePolicy
skills-lock.json 缺少 manualLanguage
```

---

## 9.2 repair 必须能恢复

```text
locale.json
style.json
plain-work-style/SKILL.md
skills-lock.json metadata
AGENTS.md Locale Policy / Voice Style Policy
Agentflow.md Locale Policy / Voice Style Policy
TDD.md Code Comment Language and Style
```

---

## 9.3 Locale mismatch 行为

如果 OS / App locale 变化：

```text
en-US -> zh-CN
```

应该：

```text
更新 locale.json
更新 workspace-manifest.json
更新 skills-lock.json agentLocale metadata
```

不应该：

```text
重写英文手册
让 skill hash mismatch
翻译 AGENTS.md
翻译 Agentflow.md
翻译 SKILL.md
```

---

# 10. Desktop / Tauri 接入

Project prepare / Agent Manual prepare 需要接收可选：

```ts
appLocale?: string
```

来源：

```ts
Intl.DateTimeFormat().resolvedOptions().locale
navigator.language
```

Tauri / Rust 侧 normalize 成 BCP 47-like string：

```text
zh_CN -> zh-CN
en_US -> en-US
ja_JP -> ja-JP
```

---

# 11. Browser Preview

Browser Preview mock 增加：

```ts
agentLocale: "zh-CN" 或 navigator.language
manualLanguage: "en"
styleId: "plain-work-style"
```

注意：

```text
Browser Preview mock 文案仍保持英文
```

因为 managed mock / policy text 统一英文。

---

# 12. Status Channel

Desktop 状态通道可以展示：

```text
Agent locale: zh-CN
Manual language: en
Voice style: plain-work-style
```

如果 locale fallback：

```text
Agent locale fallback: en-US
OS locale unavailable
```

---

# 13. 非目标

本需求不做：

```text
不做多语言手册本地化
不翻译 AGENTS.md / Agentflow.md / SKILL.md / SPEC.md / TDD.md / RELEASE.md / AUDIT.md
不调用模型翻译
不接 Claude Output Style
不接外部 writing-style-skill 项目
不接 brand voice 工具
不训练个人写作风格
不批量翻译已有代码注释
不改历史文档
不写 SPEC facts
不写 Goal Tree facts
不启动 AgentRun
不写用户源码
```

---

# 14. 建议实现文件

重点修改：

```text
crates/agent-manual/src/templates.rs
crates/agent-manual/src/model.rs
crates/agent-manual/src/manager.rs
crates/agent-manual/src/validate.rs
crates/agent-manual/src/repair.rs
crates/agent-manual/src/lock.rs
apps/desktop/src-tauri/src/commands/agent_manual.rs
apps/desktop/src-tauri/src/project_workspace/prepare.rs
apps/desktop/src/browserPreviewData.ts
apps/desktop/src/features/status-channel/statusAdapters.ts
```

建议新增：

```text
crates/agent-manual/src/locale.rs
crates/agent-manual/src/style.rs
```

---

# 15. 测试要求

至少新增 / 更新测试：

```text
prepare_records_agent_locale_and_manual_language
locale_metadata_changes_do_not_rewrite_manual_templates
validate_detects_missing_locale_state
repair_restores_locale_state
validate_detects_missing_style_state
repair_restores_style_state
validate_detects_missing_plain_work_style_skill
repair_restores_plain_work_style_skill
skills_lock_records_agent_locale_and_style_policy
code_comment_policy_is_present_in_agentflow_and_tdd_manuals
```

---

# 16. 验收标准

```text
- [ ] 新增 docs/requirements/014-1-014-2-agent-locale-and-voice-style-policy-v1.md。
- [ ] 支持读取 / 传入 OS / App locale。
- [ ] agentLocale 使用 BCP 47 locale string。
- [ ] manualLanguage 固定为 en。
- [ ] `.agentflow/define/agent/state/locale.json` 写入。
- [ ] `.agentflow/define/agent/state/style.json` 写入。
- [ ] workspace-manifest.json 记录 locale / style。
- [ ] skills-lock.json 记录 manualLanguage / agentLocale / stylePolicy。
- [ ] AGENTS.md 正文保持英文。
- [ ] AGENTS.md 包含 Locale Policy。
- [ ] AGENTS.md 包含 Voice Style Policy。
- [ ] Agentflow.md 正文保持英文。
- [ ] Agentflow.md 包含 Locale Policy。
- [ ] Agentflow.md 包含 Voice Style Policy。
- [ ] skills/SKILL.md 正文保持英文。
- [ ] 新增 plain-work-style/SKILL.md。
- [ ] plain-work-style 被 skills-lock 记录。
- [ ] TDD.md 包含 Code Comment Language and Style 规则。
- [ ] style.json styleId = plain-work-style。
- [ ] style.json appliesToCodeComments = true。
- [ ] locale 变化不导致 skills hash mismatch。
- [ ] locale 变化只更新 locale metadata。
- [ ] validate 能检测 locale.json 缺失。
- [ ] validate 能检测 style.json 缺失。
- [ ] validate 能检测 plain-work-style 缺失。
- [ ] repair 能恢复 locale.json。
- [ ] repair 能恢复 style.json。
- [ ] repair 能恢复 plain-work-style。
- [ ] Browser Preview mock 包含 agentLocale / manualLanguage / styleId。
- [ ] Status Channel 展示 agentLocale / styleId。
- [ ] 不复制第三方 skill 原文。
- [ ] 不接外部工具。
- [ ] 不调用模型。
- [ ] 不写用户源码。
- [ ] 不写 SPEC facts。
- [ ] 不写 Goal Tree facts。
- [ ] 不启动 AgentRun。
```

---

# 17. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
npm --prefix apps/desktop run preview:smoke
git diff --check
```

---

# 18. PR 说明要求

PR 描述必须说明：

```text
1. manualLanguage 为什么固定为 en。
2. agentLocale 如何检测 / 记录。
3. agentLocale 覆盖哪些用户面输出。
4. 代码注释如何跟随 agentLocale。
5. 为什么不批量翻译已有注释。
6. plain-work-style 是默认 voice policy，不是后处理工具。
7. 新增了哪些状态文件。
8. skills-lock 如何记录 locale / style。
9. locale 变化为什么不导致 skill hash mismatch。
10. 本次没有调用模型。
11. 本次没有写用户源码。
12. 本次没有写 SPEC / Goal Tree / AgentRun。
13. 验证命令和结果。
```

---

# 19. Codex 执行指令

```md
请执行 014.1 + 014.2 - Agent Locale and Voice Style Policy V1。

目标：
给 AgentFlow 的 Agent 增加默认语言和默认表达风格。AgentFlow managed manuals 统一英文；Agent 面向用户的自然语言输出必须跟随 OS / App locale；Agent 新写代码注释也必须跟随 agentLocale；所有输出必须遵守 plain-work-style：先结论、讲人话、少废话、少 AI 腔、有证据、有下一步。

必须遵守：
1. manualLanguage 固定为 en。
2. AGENTS.md 正文统一英文。
3. Agentflow.md 正文统一英文。
4. skills/SKILL.md 正文统一英文。
5. SPEC.md / TDD.md / RELEASE.md / AUDIT.md 手册正文统一英文。
6. agentLocale 必须使用 BCP 47 locale string。
7. 正常情况下 agentLocale 跟随 OS / App locale。
8. 无法检测 locale 时 fallback 到 en-US，并记录 warning。
9. Agent user-facing output must follow agentLocale。
10. Agent-authored code comments and doc comments must follow agentLocale。
11. Agent-authored code comments must also follow plain-work-style。
12. Do not mass-translate existing code comments。
13. 文件名 / JSON key / enum / API / command / path / code identifier 不翻译。
14. 新增 plain-work-style skill。
15. plain-work-style 是默认 Agent voice policy，不是可选改写工具。
16. 新增 locale.json。
17. 新增 style.json。
18. skills-lock.json 记录 manualLanguage / agentLocale / stylePolicy。
19. locale 变化不应该导致 skills hash mismatch。
20. 不复制第三方 skill 原文。
21. 不接外部 writing-style-skill / brand voice 工具。
22. 不调用模型。
23. 不写用户源码。
24. 不写 SPEC facts。
25. 不写 Goal Tree facts。
26. 不启动 AgentRun。

实现范围：
- 新增 docs/requirements/014-1-014-2-agent-locale-and-voice-style-policy-v1.md。
- 新增 locale normalization / metadata。
- 新增 style metadata。
- 更新 AGENTS.md template。
- 更新 Agentflow.md template。
- 新增 plain-work-style skill template。
- 更新 skills-lock expected template。
- 新增 `.agentflow/define/agent/state/locale.json`。
- 新增 `.agentflow/define/agent/state/style.json`。
- validate / repair 支持 locale 和 style。
- TDD.md / Build Agent 手册加入代码注释语言风格规则。
- Browser Preview mock 增加 agentLocale / manualLanguage / styleId。
- Status Channel 展示 agentLocale / styleId。
- 更新 tests / verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop run preview:smoke
- git diff --check
```

---

# 20. 完成定义

完成后，AgentFlow 应满足：

```text
工作手册统一英文
Agent 用户面输出跟随 OS / App locale
Agent 新写代码注释跟随 OS / App locale
Agent 默认风格为 plain-work-style
Agent 不再中英文混用
Agent 不再默认 AI 腔 / 官话 / 营销腔
locale 和 style 都有状态文件记录
validate / repair 能检查和恢复语言风格规则
```

最终一句话：

> **014.1 + 014.2 给 Agent 定好“语言”和“说话方式”：手册统一英文，Agent 面向用户和新写代码注释跟随系统语言，表达风格统一说人话。**
