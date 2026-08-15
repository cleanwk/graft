# 用 Rust + Tauri + Vue 复刻 Rebased：可复用开源生态调研

> 调研日期：2026-08-15（Asia/Shanghai）  
> 范围：桌面端 Git GUI；目标平台默认是 macOS、Windows、Linux。  
> 来源政策：只引用项目官方文档、官方仓库、官方 registry 页面。维护状态是本次调研快照，不是未来承诺。

## 结论先行

这件事不该被规划成“把 IntelliJ 的 Git 插件翻译成 Rust”。更可行的产品边界是：复刻用户可观察到的工作流和交互语义，后端复用系统 Git 的完整行为，前端只实现 Git 客户端特有的呈现与编排。

推荐的首版基线如下：

- **壳与 UI**：Tauri 2 + Rust stable + Vue 3 + TypeScript + Vite + Pinia；Tauri 官方明确推荐 SPA 框架配 Vite，并用系统 WebView，Windows/macOS/Linux 分别是 WebView2、WKWebView、WebKitGTK。[Tauri 前端配置](https://v2.tauri.app/start/frontend/) · [Tauri 进程模型](https://v2.tauri.app/concept/process-model/)
- **Git 行为真相源**：优先调用用户机器上的 `git` CLI（结构化输出、固定 locale、禁用 pager、严格参数数组），用 `gix`/`git2-rs` 做只读热路径或索引加速；不要在 v1 就让某个纯 Rust 引擎承担全部写操作、凭据、SSH、GPG 和 hooks 兼容性。
- **前端大数据呈现**：`@tanstack/vue-virtual` 管日志、文件树、搜索结果；提交图只绘制可见窗口；大结果分页/流式传输，避免一次把几十万提交序列化成 JSON。Tauri 官方说明大 JSON 会拖慢 IPC，并推荐 Channel 做流式数据。[Tauri 调 Rust](https://v2.tauri.app/develop/calling-rust/) · [TanStack Virtual](https://tanstack.com/virtual/v3/docs/introduction)
- **diff/编辑器**：首选 CodeMirror 6 + `@codemirror/merge`（较轻、可拆分、同时支持 side-by-side 和 unified）；只有明确需要 VS Code 级编辑体验时才引入 Monaco。CodeMirror 的 MergeView 已提供双栏、统一视图、折叠未改区和 diff 超时；Monaco 更完整但必须懒加载并主动 dispose model。[CodeMirror merge API](https://codemirror.net/docs/ref/#merge.MergeView) · [CodeMirror 更新记录](https://codemirror.net/docs/changelog/) · [Monaco 官方仓库](https://github.com/microsoft/monaco-editor)
- **终端**：`xterm.js` 只负责渲染，Rust 侧用 `portable-pty` 管真实 PTY；不要把普通 `shell` 子进程误当终端。xterm.js 提供 fit、WebGL、链接等 addon；`portable-pty` 来自 WezTerm 生态并覆盖 Unix PTY/Windows ConPTY。[xterm.js addon](https://xtermjs.org/docs/guides/using-addons/) · [portable-pty crate](https://crates.io/crates/portable-pty)
- **状态与存储**：Pinia 只放 UI/session 状态；SQLite 存仓库元数据、最近项目、索引和可迁移设置；系统 keychain/credential helper 存秘密。Pinia 是 Vue 官方推荐的新项目状态库。[Vue 状态管理](https://vuejs.org/guide/scaling-up/state-management.html)
- **安全与发布**：Tauri capability 最小授权；不要把任意 shell、文件系统或秘密暴露给 WebView。官方 updater + 每平台签名/公证；更新包签名不能替代 macOS/Windows 代码签名。[Tauri 安全模型](https://v2.tauri.app/security/) · [Tauri 发布](https://v2.tauri.app/distribute/) · [Updater](https://v2.tauri.app/plugin/updater/)

最重要的现实约束：**Tauri 官方承诺的是更小的安装包，不是“必然低内存”**。系统 WebView 仍是浏览器进程；Vue、编辑器、终端、大 DOM 和缓存都可能吃掉收益。必须在 spike 阶段以同一大型仓库、同一操作脚本实测 RSS/PSS、首屏时间、滚动帧率和操作延迟。[Tauri “What is Tauri?”](https://tauri.app/start/)

## 1. 参照物与复刻边界

本地 `/Users/kai/github/rebased` 的 README 将 Rebased 定义为“基于 IntelliJ Platform 的 Git client”，本质是保留 Git 集成并裁掉大部分 IDE 插件；截图显示核心布局是提交区、虚拟化提交日志/泳道图、选中提交的文件树与详情。[Rebased 官方 README](https://github.com/DetachHead/rebased#readme)

“功能一模一样”在动手前至少要拆成可验证的功能矩阵：仓库发现/多 root、working tree 与 shelf/stash、stage hunks/lines、commit/amend/sign、分支/tag/worktree、fetch/pull/push、rebase/cherry-pick/revert/reset、冲突解决、日志筛选、blame、submodule/LFS、hooks、终端、设置与快捷键。否则技术选型会被一个后期出现的兼容性需求推翻。

## 2. Git 引擎与 CLI 集成

| 方案 | 能复用什么 | 许可证 / 平台 / 活跃度信号 | 性能与主要风险 | 建议 |
|---|---|---|---|---|
| 系统 `git` CLI | Git 的完整 porcelain/plumbing、配置层级、credential helper、SSH/GPG、hooks、LFS/submodule/worktree，以及用户已经调好的行为 | Git 官方项目，GPL-2.0；Git 本身跨 macOS/Windows/Linux，官方持续发布。[Git 官方仓库](https://github.com/git/git) · [Git 文档](https://git-scm.com/docs) | 启进程与解析输出有成本；自然语言/进度输出并非稳定 API；命令注入、locale、pager、交互提示、取消与进程树清理必须处理。GPL 进程级调用通常比链接库更容易隔离，但发行前仍应做许可证审查 | **v1 主引擎**。优先 plumbing 和 `--format`/`-z`；参数数组执行，设置 `LC_ALL=C`、`GIT_PAGER=cat`、`GIT_TERMINAL_PROMPT=0`（需要交互时另走 askpass） |
| `git2-rs` / libgit2 | 仓库读取、revwalk、diff、index、checkout、merge 分析、remote 等；Rust API 成熟 | git2-rs 为 MIT/Apache-2.0，底层 libgit2 使用 GPL-2.0 with linking exception；支持主流桌面平台，两个官方仓库均未归档且持续维护。git2-rs 官方文档当前为 0.21，并可 vendored 构建 libgit2。[git2-rs](https://github.com/rust-lang/git2-rs) · [git2 docs](https://docs.rs/git2/latest/git2/) · [libgit2 COPYING](https://github.com/libgit2/libgit2/blob/main/COPYING) | 进程内调用避免 CLI 启动/文本解析，但没有可信依据可笼统断言更快；C FFI、OpenSSL/libssh2 和 vendored 安全更新增加负担；与用户 Git 在新特性、配置、hooks、filter、credential/SSH 行为上可能不一致 | **可选加速层**，适合稳定的读取/diff 热路径；写操作需与 CLI 结果做一致性测试 |
| `gix`（gitoxide） | 纯 Rust 的对象库、revision walk、worktree/index、diff、transport 等，支持 parallel 与 pack cache 等可裁剪 feature | MIT/Apache-2.0；官方仓库未归档、2026 年仍有 releases，Windows CI 会阻止不通过的 release。[gix 官方仓库](https://github.com/GitoxideLabs/gitoxide) · [releases](https://github.com/GitoxideLabs/gitoxide/releases) · [gix crate](https://crates.io/crates/gix) | 纯 Rust 和缓存设计是好的性能信号，但仍须真实基准；官方状态表仍把顶层 gix、gix-diff、gix-merge 等列在 initial development/usable，blame 很早期，rebase/sequencer 尚不完整。[官方 crate status](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md) | **首选只读 Rust 引擎候选**；不能独占首版写路径。先 benchmark 大仓库 log/status/diff，再逐项替换 CLI 热路径 |

不建议做一个“运行时任意切换三个引擎”的抽象层；它会把行为差异变成永久测试矩阵。应定义面向产品的窄接口（`RepositorySnapshot`、`LogPage`、`PatchSet`、`OperationPlan`），v1 只有一个权威写实现，读实现可替换。

### CLI 封装应直接复用的 Git 能力

- 日志：`git log --format=... -z`、`git for-each-ref --format=...`、`git cat-file --batch`；避免 N+1 进程。
- 状态：`git status --porcelain=v2 -z --branch` 是官方机器可读格式。[git-status](https://git-scm.com/docs/git-status)
- diff：`git diff --raw -z`/`--numstat -z` 获取清单，按需取单文件 patch；二进制、rename、submodule 不要强行文本化。[git-diff](https://git-scm.com/docs/git-diff)
- 复杂合并预演：优先复用 `git merge-tree` 等 plumbing，再决定是否落盘。[git-merge-tree](https://git-scm.com/docs/git-merge-tree)
- 长操作：为 fetch/push/rebase 建 operation actor，支持进度、凭据回调、取消、恢复后重新扫描；不要让 Vue 组件持有子进程生命周期。

先把 Git 自己已有的性能结构用好：commit-graph 可存 generation number 与 changed-path Bloom filter，MIDX 为多个 pack 建索引，内建 fsmonitor 可减少 status 扫描。它们往往比先换引擎更便宜。[commit-graph](https://git-scm.com/docs/git-commit-graph) · [MIDX](https://git-scm.com/docs/multi-pack-index) · [fsmonitor daemon](https://git-scm.com/docs/git-fsmonitor--daemon)

## 3. diff、merge 与内容编辑

| 层次 | 推荐复用 | 说明与风险 |
|---|---|---|
| Git 语义 diff | 系统 Git；或验证后的 `gix`/`git2` 只读路径 | rename/copy、submodule、二进制、属性、textconv、忽略空白等是 Git 语义，不应由前端 JS diff 库重造 |
| Rust 内联 diff 辅助 | `imara-diff` 或 `similar` | `imara-diff`（Apache-2.0）提供 Myers/Histogram、heuristics 与 fuzz，适合重型文本 diff；`similar`（Apache-2.0）适合行/词/字符内联高亮。它们都不处理完整 Git tree/index/attribute 语义。[imara-diff](https://github.com/pascalkuthe/imara-diff) · [similar](https://github.com/mitsuhiko/similar) |
| 文本展示/编辑 | CodeMirror 6 + `@codemirror/merge`（MIT） | 官方 API 有 side-by-side `MergeView` 与 unified merge；更新记录显示 merge 包持续发布，并提供 scan limit/timeout，适合防止巨大或完全不同文件卡死。[API](https://codemirror.net/docs/ref/#merge.MergeView) · [changelog](https://codemirror.net/docs/changelog/) |
| 重型替代 | Monaco Editor（MIT） | 自带 diff editor、语言能力与 worker；官方仓库仍在发布。代价是更大的 bundle/worker/model 生命周期，且内部 API 不稳定；只导入需要的 ESM feature/language 并销毁 model。[仓库](https://github.com/microsoft/monaco-editor) · [changelog](https://github.com/microsoft/monaco-editor/blob/main/CHANGELOG.md) |
| 三方 merge | `git mergetool` + 用户配置的外部工具 | 复杂语义合并与二进制文件应允许跳出到用户工具；内置 3-way editor 可后置，不要假装“两栏 diff”就是 merge solver。[git-mergetool](https://git-scm.com/docs/git-mergetool) |

首版应设置明确上限：超大文本只显示摘要/按块加载，二进制显示元数据或外部打开；不要把整个 blob 同时复制进 Rust、JSON、Pinia 和编辑器 model。

## 4. 提交图与虚拟化

提交图有两个不同问题：拓扑 lane 分配和可见区域绘制。通用 DAG layout（如 ELK）会为一般图优化，通常不符合 Git 客户端稳定、紧凑、逐行的泳道语义；`elkjs` 是仍活跃的通用 Sugiyama/layered 引擎，但许可证为 EPL-2.0 OR GPL-3.0-or-later、包更重且增量加载可能重排。[elkjs 官方仓库](https://github.com/kieler/elkjs) 老牌 `@gitgraph/js` 是 MIT，能快速做原型，但官方仓库已经 archived，且偏向构造示例图而不是导入任意大型仓库，不能作为长期核心依赖。[gitgraph.js 官方仓库](https://github.com/nicoespeon/gitgraph.js)

建议：

1. 复用 Git revwalk/topo-order 产出提交与 parent；lane 分配先用一个经过 golden fixtures 验证的窄模块，而不是引入通用自动布局引擎。
2. `@tanstack/vue-virtual`（MIT）只挂载可见提交行；它是 headless，可保留完整 DOM/CSS/SVG/canvas 控制，并支持纵向/横向虚拟化。[官方文档](https://tanstack.com/virtual/latest/docs/framework/vue) · [官方仓库](https://github.com/TanStack/virtual)
3. 图线用单个 overlay canvas 或分段 SVG 绘可见范围，文本行仍用 DOM 保证选择、键盘和可访问性。不要每行创建大量 SVG path。
4. 数据按 cursor/page 拉取并保留有限窗口；过滤条件变化时可取消旧任务。先在 Linux/macOS/Windows 的实际 WebView 上测 100k/1M commit 仓库。

## 5. 终端

| 组件 | 许可证 / 活跃度 / 平台 | 用法与风险 |
|---|---|---|
| `@xterm/xterm` | MIT；VS Code 使用的终端前端，官方 release 与 6.x 文档持续更新；运行于 WebView。[下载与 releases](https://xtermjs.org/docs/guides/download/) · [API](https://xtermjs.org/docs/) | 只做 VT 渲染和输入；配 `addon-fit`，可选 WebGL/Unicode/Web Links。链接打开要走受控 opener，并保留 Ctrl/Cmd 修饰键等防误触策略。[链接安全](https://xtermjs.org/docs/guides/link-handling/) |
| `portable-pty` | MIT；WezTerm 官方 workspace 中的 Rust crate，Unix PTY + Windows ConPTY。[crate](https://crates.io/crates/portable-pty) · [WezTerm 仓库](https://github.com/wez/wezterm) | Rust 侧拥有 PTY、resize、子进程组和退出；维护 backpressure，退出时清理整个会话。该项目 release 节奏较慢且主要由 WezTerm 驱动，需单独做 Windows 信号/编码/关闭测试 |

Tauri 官方 `shell` 插件适合受 scope 限制的普通命令/sidecar，不替代 PTY。[Shell plugin](https://v2.tauri.app/plugin/shell/) 终端能力应是单独的 Rust 模块和 capability，不给前端“执行任意字符串”的接口。

## 6. 凭据、SSH、GPG 与敏感数据

兼容性优先级最高的路径是让系统 Git 继续编排这些组件：

- Credential：遵循 `credential.helper`，包括 Git Credential Manager（跨平台、MIT）和系统已有 helper；应用用 `git credential fill/approve/reject` 协议，不自己保存密码。[Git credential 文档](https://git-scm.com/docs/gitcredentials) · [Git Credential Manager](https://github.com/git-ecosystem/git-credential-manager)
- SSH：尊重 `GIT_SSH_COMMAND`、`core.sshCommand`、`~/.ssh/config`、agent 和 known_hosts；不要默认内嵌另一套 SSH 配置解释器。[Git 环境变量](https://git-scm.com/book/en/v2/Git-Internals-Environment-Variables) · [git-config core.sshCommand](https://git-scm.com/docs/git-config#Documentation/git-config.txt-coresshCommand)
- commit/tag 签名：让 Git 调用用户配置的 `gpg.format`/`user.signingKey` 与 GPG/SSH/X.509 工具；UI 负责状态与 prompt，不处理私钥。[git-config 签名选项](https://git-scm.com/docs/git-config#Documentation/git-config.txt-gpgformat)
- 必须保存的应用秘密：优先系统 keychain；Tauri 官方 Stronghold 是加密数据库，但它不是系统 credential helper 的透明替代品，移动平台覆盖也与桌面不同。[Stronghold plugin](https://v2.tauri.app/plugin/stronghold/)
- 如果产品未来有**自身** OAuth token（不是 Git 已有凭据），`keyring-rs`（MIT/Apache-2.0）可统一访问 macOS/Windows/*nix 原生 secure store，官方仓库 2026 年仍在维护；按目标平台只编译对应 store crate。[keyring-rs](https://github.com/open-source-cooperative/keyring-rs)

风险点：GUI 启动时环境变量常与终端不同；macOS 的 PATH/agent socket、Windows Git/MSYS/PowerShell、Linux Secret Service 都要做真实安装环境测试。Askpass helper 必须防 prompt spoofing，并在展示 remote host/key fingerprint 时清楚标识来源。

## 7. 文件监听、缓存与持久化

| 需求 | 推荐包 | 许可证 / 平台 / 活跃度 | 注意事项 |
|---|---|---|---|
| 仓库变化监听 | Rust `notify` | CC0-1.0；封装各平台原生 watcher，有活跃 releases。[官方仓库](https://github.com/notify-rs/notify) · [crate](https://crates.io/crates/notify) | watcher 事件会重复、乱序、溢出；只把它当“可能变了”的 invalidation signal，debounce 后重新跑 status/ref scan。网络盘、WSL、容器挂载要允许 polling fallback |
| 简单设置 | Tauri `store` + `window-state` | 官方 plugins workspace，MIT/Apache-2.0；桌面三平台支持，持续集中发布。[插件清单与平台矩阵](https://github.com/tauri-apps/plugins-workspace) | 适合主题、最近仓库、窗格尺寸，不适合查询型数据与秘密 |
| 查询型本地数据 | SQLite：Rust 侧 `rusqlite` 或 `sqlx`；也可用官方 `tauri-plugin-sql` | rusqlite MIT；sqlx MIT/Apache-2.0；SQLite public domain。三平台成熟。[rusqlite](https://github.com/rusqlite/rusqlite) · [sqlx](https://github.com/launchbadge/sqlx) · [Tauri SQL](https://v2.tauri.app/plugin/sql/) | 推荐由 Rust repository service 独占 schema/migration，不把任意 SQL 暴露给 WebView。Git 对象本身不复制进数据库，缓存必须可丢弃/重建 |
| UI 状态 | Pinia | MIT、Vue core team 维护、Vue 官方推荐；跨 WebView。[Pinia](https://pinia.vuejs.org/) · [Vue 建议](https://vuejs.org/guide/scaling-up/state-management.html) | 只放可序列化的视图/session 状态；大 blob、commit 全量、PTY handle、子进程不进 store |

## 8. Tauri 2 与官方插件

Tauri / 官方 plugin workspace 采用 MIT 或 Apache-2.0；官方插件表给出了桌面和移动平台矩阵，仓库与 releases 在 2026 年仍有更新。[Tauri 仓库](https://github.com/tauri-apps/tauri) · [官方插件仓库](https://github.com/tauri-apps/plugins-workspace) · [插件 releases](https://github.com/tauri-apps/plugins-workspace/releases)

首版建议启用：

- `dialog`：打开仓库/保存 patch；`opener`：受控打开 URL/文件；`clipboard-manager`；`os`。
- `single-instance`：二次启动时把 repo/path 路由到现有窗口。
- `window-state`：窗口位置/大小；复杂 pane layout 放应用设置。
- `log`：Rust + frontend 可观测性；日志必须脱敏 remote URL token、命令环境与凭据。
- `updater` + `process`：检查/安装更新并重启。
- `shell`：只用于白名单 sidecar 或明确 scope 的外部工具；Git 主流程更适合 Rust 后端直接 `Command`，避免给 WebView 通用 shell 权限。
- `fs`：尽量不向前端开放整个仓库；由 Rust 命令返回必要数据。如果确需 watch，官方 fs plugin 也有 `watch` feature。[Tauri fs](https://v2.tauri.app/plugin/file-system/)

Tauri capability 应按窗口和命令分组、deny-by-default；官方说明 capability 控制 WebView 对 IPC 的细粒度访问。核心进程保留 repo path 校验、symlink/canonicalization、命令参数构造与秘密。[Capability 配置参考](https://v2.tauri.app/reference/config/#capability) · [Tauri 安全](https://v2.tauri.app/security/)

## 9. Vue UI、布局与视觉栈

| 包 | 建议 | 许可证 / 活跃度 / 风险 |
|---|---|---|
| Vue 3 + TypeScript + Vite | 基线；Composition API，路由可选（单主窗口不一定需要 Vue Router） | Vue MIT 且持续发布；Tauri 官方推荐 Vite 做 SPA。[Vue 仓库](https://github.com/vuejs/core) · [Tauri Vite](https://v2.tauri.app/start/frontend/vite/) |
| Reka UI | 用于菜单、popover、dialog、tabs、tooltip、context menu、select 等无样式 primitives | MIT、活跃；官方说明遵循 WAI-ARIA、处理焦点/键盘且可 tree-shake。它不提供成品视觉，需要自建 design tokens。[介绍](https://www.reka-ui.com/docs/overview/introduction) · [可访问性](https://www.reka-ui.com/docs/overview/accessibility) · [仓库](https://github.com/unovue/reka-ui) |
| Tailwind CSS + CSS variables | 编译期生成样式，快速建立 compact/dark/light/high-contrast token；也可只用原生 CSS modules | MIT、活跃。[官方仓库](https://github.com/tailwindlabs/tailwindcss) 平台风险主要来自不同系统 WebView 的 CSS 差异，需三平台视觉回归 |
| shadcn-vue | 可选的“复制到仓库的成品组件”起点，不作为不可控黑盒依赖 | MIT、2026 仍在发版；基于 Reka/Tailwind，得到漂亮默认值但代码归项目维护。[官方仓库/releases](https://github.com/unovue/shadcn-vue/releases) |
| Splitpanes | 左/中/右与上下可调 pane | MIT，Vue 3，v4 增加键盘/ARIA 与多项 resize 修复。[官方仓库](https://github.com/antoniandre/splitpanes) 仍需测试嵌套、最小尺寸和高 DPI |
| TanStack Virtual | 日志、文件树、搜索、分支列表 | MIT、活跃，headless，100% 控制 markup/style。[官方文档](https://tanstack.com/virtual/v3/docs/introduction) |
| Lucide Vue | 通用线性图标 | ISC（部分 Feather 衍生图标 MIT）、持续发布；注意随发行物保留许可声明。[官方仓库](https://github.com/lucide-icons/lucide) · [LICENSE](https://github.com/lucide-icons/lucide/blob/main/LICENSE) |

不要把“UI 更漂亮”等同于加入动画组件库。Git 客户端的视觉质量主要来自信息密度、对齐、键盘焦点、选中态、graph 颜色稳定、空/加载/错误态与三平台字体渲染。动画只用于解释状态变化，并遵循 reduced-motion。

## 10. 测试、打包与自动更新

### 测试金字塔

1. **Rust unit/golden**：命令构造、porcelain parser、路径与 URL 脱敏、lane layout；用 fixture repos 覆盖 merge/octopus、rename、submodule、worktree、冲突、非 UTF-8 path。
2. **Rust integration**：临时目录中调用真实 Git；矩阵覆盖平台 Git 版本与 SHA-1/SHA-256（在支持范围内）。不要 mock 掉真正想兼容的 Git 行为。
3. **Vue component**：Vitest + Vue Test Utils。两者均 MIT；Vitest 复用 Vite 配置且发布活跃，VTU 是 Vue 3 官方测试工具。[Vitest](https://github.com/vitest-dev/vitest) · [Vue Test Utils](https://test-utils.vuejs.org/)
4. **桌面 E2E**：Tauri 官方当前推荐 WebdriverIO 的 Tauri service，可用 embedded provider 覆盖 Windows/Linux/macOS，并支持 IPC mock、Rust 命令执行和日志；直接 `tauri-driver` 仍仅 Windows/Linux。[Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/) CI 需每平台至少一条真实 Git smoke flow。
5. **性能回归**：固定仓库快照与操作脚本，记录 cold/warm open、100k log 首屏、滚动 dropped frames、status、diff、fetch UI 响应、idle/active RSS/PSS、安装包尺寸。结果按平台分开，不用单一数字营销。

### 打包与更新

- Tauri CLI 可构建平台 installer；Linux 支持 AppImage/deb/rpm 等，macOS app/dmg/App Store，Windows MSI/NSIS/Store。大多数平台需要代码签名，macOS 站外分发还需 notarization。[Tauri 发布矩阵](https://v2.tauri.app/distribute/)
- 用 `tauri-action` 做三平台矩阵构建/上传，但签名证书与 updater 私钥只能在受保护 CI secret 中。[Tauri GitHub pipeline](https://v2.tauri.app/distribute/pipelines/github/)
- 官方 updater 会生成/验证 `.sig` 更新制品；启用 `createUpdaterArtifacts`，使用 HTTPS endpoint，设计 staged rollout/rollback/最低版本策略。[Updater](https://v2.tauri.app/plugin/updater/)
- Windows 默认依赖系统/引导安装 WebView2；离线或固定 runtime 会让 installer 增加约 127MB/180MB，直接抵消“小包”优势。[Windows WebView2 安装选项](https://v2.tauri.app/distribute/windows-installer/#webview2-installation-options)

## 11. 推荐架构与复用边界

```text
Vue view (虚拟列表 / graph overlay / CodeMirror / xterm)
        │  typed invoke + Channel；分页、取消、背压
        ▼
Tauri application service
  ├─ RepositoryService       仓库快照、status、log、diff
  ├─ OperationService        fetch/push/rebase/... 状态机
  ├─ CredentialBroker        askpass/credential helper，不持有私钥
  ├─ TerminalService         portable-pty 会话
  ├─ WatchService            notify → debounce → invalidate
  └─ Settings/Cache          SQLite + window/store
        │
        ├─ 系统 git（权威写行为与完整兼容）
        └─ gix 或 git2（经过 benchmark 的只读热路径，可选）
```

这里值得自己写的不是 Git 算法，而是产品特有的薄层：类型化用例接口、operation 状态机、提交 lane 的稳定映射、虚拟窗口图形、跨平台错误翻译和可恢复 UX。Git 协议、对象格式、diff 语义、凭据、SSH、GPG、PTY、编辑器、虚拟列表、对话框、更新器都应复用现成项目。

## 12. 关键风险与决策门

| 决策门 | 必须先证明什么 | 否则的后果 |
|---|---|---|
| CLI-only vs hybrid | 在 1M commit/大 monorepo 上，批处理 CLI 能否达到目标；gix/git2 是否实测更快且语义一致 | 过早引擎抽象，或后期大改 |
| CodeMirror vs Monaco | stage hunk/line、unified/side-by-side、语法高亮、巨大文件、IME、内存是否达标 | Monaco 抬高常驻内存，或 CodeMirror 后期补功能 |
| 内建 terminal 是否首发 | 用户场景是否真的依赖；Windows ConPTY/macOS shell/Linux Wayland 的测试成本 | 终端成为与 Git 无关的大型子产品 |
| 内建 3-way merge 是否首发 | 冲突 UX、rerere、binary/submodule、ours/theirs/base 语义是否完整 | 数据损坏或用户误判 |
| Linux 支持等级 | 目标发行版/WebKitGTK 版本、Wayland/X11、secret service、package formats | “支持 Linux”不可验证 |
| “功能一模一样” | Rebased 功能清单、快捷键、上下文菜单、错误/取消/恢复语义的验收基线 | 无限 scope，无法判断完成 |

## 13. 建议的两阶段验证（尚不是实施计划）

### Spike A：兼容性与性能（先做）

- 三平台打开同一 fixture repo；实现 status porcelain v2、分页 log、单文件 diff、refs。
- 以系统 Git 为 baseline，同时对 gix 与 git2 的只读热路径跑相同 benchmark 和 golden output。
- Vue 只做最小虚拟日志 + graph overlay + CodeMirror diff；测冷启动、RSS/PSS、滚动和 IPC payload。
- 验证 fetch 的 credential helper/SSH agent/取消；验证签名 commit 只通过系统 Git。

### Spike B：桌面分发闭环

- 三平台 CI build、签名、安装、打开 repo、自动更新到下一版本、回滚演练。
- Windows 覆盖没有 WebView2/无网场景策略；macOS 覆盖 notarization；Linux 明确 package/发行版支持表。

两个 spike 通过后，才适合冻结正式架构与里程碑。

## 14. 许可证与供应链摘要

- 推荐前端主栈大多是 MIT/ISC；Tauri/git2/gix 常见为 MIT OR Apache-2.0；`notify` 是 CC0-1.0；SQLite 是 public domain。
- 系统 Git 是 GPL-2.0；libgit2 是 GPL-2.0 with linking exception。**本表不是法律意见**，发布前要生成 Rust/JS/sidecar 的 SBOM 与 NOTICE，并由项目负责人确认动态/静态链接和再分发边界。
- 锁定 Cargo/npm lockfile；CI 使用 `cargo audit`/`cargo deny`、npm provenance/审计与 release 制品哈希；社区 Tauri plugin 在采用前按维护者、release、权限面、native code 和退出策略审计。Tauri 官方插件集中在同一 workspace 并有明确平台矩阵，应优先于同功能社区插件。[官方插件 workspace](https://github.com/tauri-apps/plugins-workspace)

## 15. 待产品访谈确认的问题

1. “一模一样”是对齐 Rebased 当前全部 IntelliJ Git 功能，还是先对齐截图中的 5–10 个日常工作流？
2. 首发必须同时支持 macOS/Windows/Linux 吗？最低 OS/发行版是什么？
3. 是否接受依赖用户安装 Git？若不接受，打包 Git 会显著改变体积、更新、安全与 GPL 合规工作。
4. GitHub/GitLab OAuth 是首发要求，还是尊重现有 credential helper 即可？
5. 内置终端、编辑器、3-way merge、LFS、submodule、worktree、shelf/changelist 各自是否 P0？
6. “更小/更快”的量化门槛是什么：安装包、idle RSS、峰值 RSS、启动、100k log、status 还是 push 体验？比较基准机器与仓库是什么？
7. 是否要兼容 Rebased/JetBrains 的快捷键与设置迁移，还是只复刻功能和信息架构？
8. 应用是纯本地开源，还是会有账号、遥测、云同步或商业签名服务？这会改变威胁模型与许可/隐私范围。
