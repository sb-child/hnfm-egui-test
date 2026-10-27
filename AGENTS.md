# hnfm-egui-test

egui 自定义 Material 3 组件测试代码，正在从纯 M3 迁移到 `dux-qml-common` 实现，未来会移植到另一项目(https://github.com/sb-child/HoshinekoFM)

## 资源

参考实现：

- 主要参考：[`duxlabs-dev/dux-qml-common`](https://github.com/duxlabs-dev/dux-qml-common)，固定 commit `c6e4a6bce24b804688225b0889410d2ec0ae06d7`
  - 组件：`DCommon/Widgets/`
  - 样式：`DCommon/Common/Style.qml`
  - 形状：`DCommon/Common/Shape.js`、`SHAPES.md`
  - 应用入口：[`dux-shell/quickshell`](https://github.com/duxlabs-dev/dux-shell/tree/master/quickshell)
- 项目正在从纯 M3 迁移到上面的实现。组件尺寸、圆角、state layer、动效优先对照 `dux-qml-common`。

material3 规范：

- github_search_code mcp: `query=xxx, repo:androidx/androidx`
- 组件实现：`https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/`
- 设计 token：`https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/`

规范用于理解语义和可访问性；与 `dux-qml-common` 的实际行为冲突时，以 `dux-qml-common` 为准。

## 组件契约

原则：mechanics 可以隐藏，policy 必须显式。

- 隐藏：state layer 怎么叠、动画 ID 怎么派生、圆角怎么命中、颜色怎么插值。
- 显式：尺寸、变体、间距、动效、主题、ID。
- 不做：hover 驱动尺寸、label 派生 ID、全局主题单例、隐藏两趟布局。

### 尺寸

- 宽度用 `Hug` / `Fill` / `Fixed`，高度用 `Hug` / `Fixed`。
- 固定尺寸组件直接用 token；内容尺寸组件先 measure 再 `allocate_exact_size`；Fill 用 `ui.available_width()`。
- 容器只负责排列，不做隐藏 measure pass。需要等宽、右对齐时用显式 helper，不写通用 flex 引擎。
- 文本测量统一走 `measure_text`，不要从 `Response` 反推 Hug 宽度。

### 动画

- `Motion` 是数据：时长 + 可采样的 `Curve`。
- 一个 `animate` driver；目标值是 f32，不是 bool。
- 组件自持交互状态和动画，调用方只传业务状态。
- 动画只改绘制：颜色、圆角、alpha、scale、ripple。
- 不要动画：分配宽高、兄弟节点位置、滚动内容高度。展开/收起由显式 `open: bool` 驱动。
- 动画 ID 从调用方给的稳定 key 派生，并在文档写明。

### 调用形态

- 优先 `M3Ui` + typed builders；主题和动效偏好每帧显式传入，不用全局单例。
- 现有 `color::access` / `set_global_scheme` 是迁移遗留；新组件不要依赖全局主题，迁移时改为显式传入。
- 组件方法返回可判断点击的响应（优先 `Response`；需要圆角命中时用显式 `Hit`），保留 `m.ui()` 作为原生 egui escape hatch。
- 每个组件在文档里写清“尺寸 + 动效 + 命中”契约。

### 反魔法自查

- [ ] hover/press/focus 不影响 allocated size
- [ ] 没有从 label 派生 ID；key 由调用方给
- [ ] 没有隐藏的 measure pass / 二次布局
- [ ] 主题、动效偏好显式传入，没有全局单例
- [ ] 没有 bool 参数控制语义，改用 enum
- [ ] 动画只改绘制，除了显式文档化的展开/收起
- [ ] 动画 ID 由显式 key 派生，且文档说明了
- [ ] 每个组件有“尺寸 + 动效 + 命中”契约表
- [ ] `MotionToken::None` / `reduce_motion` 能一键关掉动画
- [ ] 有 debug overlay 能看到 allocated rect、命中形状、动画值

### 实现顺序

1. `tokens.rs`
2. `motion.rs`
3. `interact.rs`
4. `text.rs`
5. `button.rs`
6. 2 到 3 个真实调用点
7. 再决定 `ListItem` / `IconButton` / `NavRail` 是否复用同一套抽象

## 准则

先检查会不会破坏别的代码逻辑和功能。会的话跟我说！！

**写完问自己这些问题（代码可读性和健壮性）**

1. 整洁度，文档完善度

- 注释，文档?
- 代码本身足够描述代码的功能吗
- 人容易在有限的视野里看明白代码逻辑吗？（根据调用链优化代码布局，文件树等。注释说明这个类型/函数会在哪里被使用）
- 注释/文档清晰吗？每个函数和文件顶部都有注释和文档吗？有临时注释要清除吗？
- 是否存在`-----------`/`==========`/`↔ → —`等字符(去掉或者用键盘能打的字符替代)？
- 是否有未明确注释的不明意义行为(或者可以理清代码让它更清晰吗)
- 函数很长吗？不同的功能可以拆分到不同的文件吗？
- 大缩进(>=4 tabs)能拆分吗？能展平吗？

2. 日志 / tracing

- 有多次出现的字段吗:
```rust
// 多余的 worker id
let worker_span = tracing::info_span!("worker", id = %cmd.fs_worker_id);
spawn(async { /* ... */ warn!("worker id {worker_id} crashed");}).instrument(worker_span);

// 改进后
let worker_span = tracing::info_span!("worker", id = %cmd.fs_worker_id);
spawn(async { /* ... */ warn!("crashed");}).instrument(worker_span);

// 多余的 service worker 和 worker_id
#[instrument(name = "service worker")]
async fn start_worker(worker_id: u64) { debug!("service worker: id={worker_id} starting"); /* ... */ }

// 改进后
#[instrument(name = "service worker")]
async fn start_worker(worker_id: u64) { debug!("starting"); /* ... */ }
```
- `tracing::Instrument` 应该注入到程序的所有地方(spawn, 函数块)。在关键区域注明 `tracing::info_span!()`
- span 往下传递了吗

3. hack，workaround, hotfix

- 不必要的全局变量，不必要的硬编码，不必要的sleep
- 意义不明的 `bool` 参数 -> 语义清晰的 `enum`

4. 错误处理

- `Result<T, String>` → `https://docs.rs/snafu/latest/snafu/`
- 所有的spawn，处理过里面的错误吗，能把里面的错误传回handle吗
- `.unwrap()` safety? `unsafe { ... }` safety? 调用了可能`panic!()`的函数？如果spawn内部panic了怎么办？
- 不可恢复错误 `Err` 能传回程序入口点并优雅处理吗？如果在保持优雅的情况下不能，那可以`tracing::error!()`吗。

5. 异步代码和性能

- 不必要的copy？async里跑阻塞代码？spawn_blocking 里面如果有循环，能在需要cancel时打断吗？
- async里的await能一起执行而不是挨个await吗？sync会卡多久？如果需要，sync能cancel吗？
- mutex竞争？能换成更好的架构吗(channel)？

**material3 组件相关**

1. 对照 `dux-qml-common` 的实现，尺寸和行为一致吗？
2. 对照 AndroidX M3 规范，语义和可访问性一致吗？
3. 图省事没处理好动画？
4. 考虑 edge case 了吗？
