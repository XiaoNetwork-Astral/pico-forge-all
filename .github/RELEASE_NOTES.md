Updates:

- Bundled picotool in the Windows x64 package, fixed update-mode detection and premature timeouts, and kept controls disabled until the device reconnects after flashing
- Fixed signing and inspection triggering update mode and duplicate signing-key checks; improved firmware validation and update confirmation
- Added Pico All management for PIV, OpenPGP, SmartCard-HSM and Audit
- Added searchable key, object, slot and security-event lists; HSM objects accept text or files and receive IDs automatically
- Added English and Simplified Chinese, IANA time zones and calendar timestamps for Audit events
- Improved PIN validation and factory-default handling, reset flows and per-status light controls

---

更新内容：

- Windows x64 便携包内置 picotool，修复更新模式切换失败及提前超时；刷写后等待设备重连，再恢复按钮操作
- 修复签名和检查意外触发更新模式、签名密钥重复匹配，改进固件校验和刷写确认
- 新增 Pico All 的 PIV、OpenPGP、SmartCard-HSM 和审计管理
- 密钥、对象、槽位和安全事件可搜索；HSM 对象支持文本或文件导入，ID 自动分配
- 新增中英文界面、IANA 时区列表和审计事件日历时间
- 改进 PIN 必填校验与默认值处理、重置流程和各状态的指示灯设置
