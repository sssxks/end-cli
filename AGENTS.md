修改rust代码后跑，没改rust代码不用跑：`just done`, `scripts/build_web_wasm.sh`。
修改前端代码后在`web`目录下跑：`npm run check` 检查类型和 `npm run test:e2e` 运行e2e测试。

Tauri 桌面端开发：在`web`目录下跑 `npm run dev:tauri` 启动 Tauri 开发模式，`npm run build:tauri` 构建桌面端安装包。
Tauri 后端代码在 `web/src-tauri/src/lib.rs`，通过 Tauri IPC 调用 `end-web` crate 的纯 Rust API。

开发这个项目需使用 rust-dev skill。
