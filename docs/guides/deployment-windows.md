# Windows 部署指南

## 环境准备

### 1. 安装 Rust（必需）

```powershell
# 下载并运行 Rust 安装器
# 访问 https://rustup.rs/ 下载 rustup-init.exe
# 或直接运行：
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe

# 安装完成后，重启终端并验证
rustc --version
cargo --version
```

### 2. 安装 Protocol Buffers 编译器（必需）

```powershell
# 使用 Chocolatey 安装
choco install protoc -y

# 验证安装
protoc --version
```

如果没有 Chocolatey，可以手动安装：
1. 访问 https://github.com/protocolbuffers/protobuf/releases
2. 下载 `protoc-xx.x-win64.zip`
3. 解压到 `C:\protoc`
4. 添加 `C:\protoc\bin` 到系统 PATH

### 3. 启用 VBSCRIPT 功能（MSI 构建必需）

```powershell
# 打开 Windows 功能
# 设置 → 应用 → 可选功能 → 更多 Windows 功能 → VBSCRIPT
# 或使用 PowerShell（需要管理员权限）：
Enable-WindowsOptionalFeature -Online -FeatureName "VBSCRIPT" -All
```

### 4. 安装项目依赖

```bash
# 安装 Node.js 依赖
npm install

# Rust 依赖会在构建时自动下载
```

## 本地构建

### 开发模式

```bash
# 启动开发服务器（热重载）
npm run tauri dev
```

### 生产构建

```bash
# 构建 Windows .msi 安装包
npm run tauri build
```

构建产物位置：
- MSI 安装包：`src-tauri\target\release\bundle\msi\OpenCode LLM Wiki_0.0.1_x64_en-US.msi`
- NSIS 安装包：`src-tauri\target\release\bundle\nsis\OpenCode LLM Wiki_0.0.1_x64-setup.exe`

## GitHub Actions 自动构建

### 方式 1：推送标签触发发布

```bash
# 创建并推送版本标签
git tag v0.0.1
git push origin v0.0.1
```

GitHub Actions 会自动：
1. 构建 Windows/macOS/Linux 安装包
2. 创建 GitHub Release
3. 上传所有安装包到 Release

### 方式 2：手动触发构建（测试用）

1. 访问 GitHub 仓库的 Actions 页面
2. 选择 "Build & Release" workflow
3. 点击 "Run workflow"
4. 选择分支并运行

这种方式会构建安装包但不创建 Release，构建产物作为 workflow artifacts 保存 14 天。

## 代码签名（可选但推荐）

未签名的安装包会触发 Windows SmartScreen 警告。

### 获取代码签名证书

1. **OV 证书**（组织验证）：约 $100-300/年
   - 提供商：DigiCert, Sectigo, GlobalSign
   - 需要企业验证
   - 初期会有 SmartScreen 警告，需要积累信誉

2. **EV 证书**（扩展验证）：约 $300-500/年
   - 立即获得 SmartScreen 信誉
   - 需要硬件 USB token
   - 推荐用于商业发布

### 配置代码签名

#### 本地签名（OV 证书）

1. 将证书导入 Windows 证书存储：

```powershell
# 导入 .pfx 证书
$password = ConvertTo-SecureString -String "YOUR_PASSWORD" -Force -AsPlainText
Import-PfxCertificate -FilePath certificate.pfx -CertStoreLocation Cert:\CurrentUser\My -Password $password

# 获取证书指纹
# 打开 certmgr.msc → 个人 → 证书 → 双击证书 → 详细信息 → 指纹
```

2. 配置 `src-tauri/tauri.conf.json`：

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "A1B1A2B2A3B3A4B4A5B5A6B6A7B7A8B8A9B9A0B0",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.comodoca.com"
    }
  }
}
```

#### GitHub Actions 签名

1. 将证书转换为 Base64：

```powershell
certutil -encode certificate.pfx base64cert.txt
# 复制 base64cert.txt 的内容（去掉头尾的 BEGIN/END 行）
```

2. 在 GitHub 仓库添加 Secrets：
   - `WINDOWS_CERTIFICATE`：Base64 编码的证书内容
   - `WINDOWS_CERTIFICATE_PASSWORD`：证书密码

3. 证书会在 GitHub Actions 构建时自动使用（已配置在 `.github/workflows/build.yml`）

## WebView2 运行时

Windows 10 (1803+) 和 Windows 11 已预装 WebView2。

如需支持离线安装或旧版本 Windows，修改 `src-tauri/tauri.conf.json`：

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "embedBootstrapper"  // 嵌入 1.8MB 安装器
        // 或 "offlineInstaller"     // 嵌入 127MB 完整安装器
      }
    }
  }
}
```

## 常见问题

### 1. `light.exe` 失败

**原因**：VBSCRIPT 功能未启用

**解决**：启用 VBSCRIPT 功能（见上文第 3 步）

### 2. `protoc` 未找到

**原因**：Protocol Buffers 编译器未安装或不在 PATH

**解决**：
```powershell
choco install protoc -y
# 或手动下载并添加到 PATH
```

### 3. 链接错误（LNK1181）

**原因**：Visual Studio Build Tools 未安装

**解决**：
1. 下载 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. 安装时勾选 "Desktop development with C++"

### 4. SmartScreen 警告

**原因**：安装包未签名或证书信誉不足

**解决**：
- 短期：用户可以点击 "更多信息" → "仍要运行"
- 长期：购买代码签名证书并配置签名

## 构建优化

当前配置已优化：
- **LTO**：链接时优化，减小体积
- **Strip**：移除调试符号
- **opt-level = "s"**：优化体积而非速度
- **codegen-units = 1**：单个代码生成单元，更好的优化

预期安装包大小：约 50-80MB（取决于 WebView2 安装模式）

## 发布检查清单

- [ ] 更新版本号（package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json）
- [ ] 更新 CHANGELOG.md
- [ ] 本地测试构建：`npm run tauri build`
- [ ] 在干净的 Windows VM 上测试安装
- [ ] 推送版本标签：`git tag v0.0.x && git push origin v0.0.x`
- [ ] 验证 GitHub Actions 构建成功
- [ ] 下载并测试 Release 中的安装包
- [ ] 更新 README.md 中的下载链接

## 参考资源

- [Tauri v2 官方文档](https://v2.tauri.app/)
- [Windows 前置条件](https://v2.tauri.app/start/prerequisites/#windows)
- [Windows 代码签名](https://v2.tauri.app/distribute/sign/windows/)
- [GitHub Actions 配置](https://v2.tauri.app/distribute/pipelines/github/)
