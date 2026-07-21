# hnfm-egui-test

egui 自定义 material 3 组件测试代码，未来会移植到另一项目(https://github.com/sb-child/HoshinekoFM)

## 资源

material3 规范：

- github_search_code mcp: `query=xxx, repo:androidx/androidx`
- github: `https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/xxx.kt`

动 material3 组件先看规范，以规范为准。

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

1. 对照 androidx 代码，跟标准一致吗？
2. 图省事没处理好动画？
3. 考虑 edge case 了吗？
