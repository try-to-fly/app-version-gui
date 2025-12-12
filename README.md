# App Version GUI

跨平台软件版本监控工具，用于追踪软件最新版本并与本地版本对比。

## 功能

- 📦 监控软件最新版本（GitHub Release / Tags / Homebrew）
- 🔍 自动检测本地安装版本
- ⚡ 版本对比，一键刷新
- 🎨 明暗主题切换
- ⏰ 定时自动刷新

## 技术栈

- **前端**: React 19 + TypeScript + Tailwind CSS
- **桌面**: Tauri 2
- **状态管理**: Zustand
- **UI 组件**: Radix UI

## 开发

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建应用
pnpm tauri build
```

## 使用

1. 点击「添加软件」
2. 选择数据源类型（GitHub Release / Tags / Homebrew）
3. 填写软件标识符（如 `owner/repo` 或 `formula-name`）
4. （可选）配置本地版本检测命令
5. 保存后自动获取版本信息

## 设置

- **GitHub Token**: 配置后提高 API 请求限额
- **缓存策略**: 设置缓存有效期和自动刷新间隔
- **主题模式**: 浅色 / 深色 / 跟随系统

## License

MIT
