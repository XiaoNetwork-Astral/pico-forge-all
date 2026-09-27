Updates:

- Verify Pico All hardware settings after writing; fix manufacturer refresh, empty-name defaults and Pico All LED colour-order encoding
- Reject invalid numeric settings and unsupported legacy FIDO fields instead of silently ignoring them; use the updated v8.3 firmware for manufacturer editing
- Show inline errors for invalid PIV management keys and OpenPGP PIN lengths; use the standard Generate button style
- Move feature switches to Passkeys, Accounts, Slots, PIV, OpenPGP and HSM, with descriptions for U2F and OTP; HSM switching requires the updated v8.3 firmware
- Set initial HSM PINs directly on its PIN card, without a setup popup; correct Audit log-reading confirmation prompts
- Windows x86/x64 portable ZIPs and installers include picotool

---

更新内容：

- Pico All 硬件配置写入后逐项核对；修复制造商名称刷新、名称清空恢复默认及 LED 色序编号
- 数值输入无效、旧版 FIDO 不支持的字段会明确报错；制造商编辑需配合本次更新的 v8.3 固件
- PIV 管理密钥格式及 OpenPGP PIN 长度错误直接显示在输入框下方；统一生成按钮样式
- 功能开关移入通行密钥、账户、槽位、PIV、OpenPGP 和 HSM 页面；补充 U2F 和 OTP 的用途说明；HSM 开关需配合更新后的 v8.3 固件
- HSM 首次 PIN 设置直接放在 PIN 卡片内，不再弹出初始化窗口；修正审计读取日志的按键提示
- 提供内置 picotool 的 Windows x86／x64 绿色版和安装版
