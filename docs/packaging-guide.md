# Oriv Windows 打包与引导程序制作流程（PowerShell）

## 准备
- 安装 WiX 3.14（与 cargo-wix 兼容），bin 路径示例：`C:\Program Files (x86)\WiX Toolset v3.14\bin`
- 已安装 Windows SDK（包含 signtool），示例路径：`C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe`
- PowerShell 下将 WiX bin 临时加入 PATH（当前会话）
  ```powershell
  $env:PATH="C:\Program Files (x86)\WiX Toolset v3.14\bin;$env:PATH"
  ```

## 构建应用与 MSI
1) 生成发布版可执行文件
   ```powershell
   cargo build --release
   ```
2) 生成 MSI（产物：`target/wix/Oriv-0.1.0-x86_64.msi`）
   ```powershell
   cargo wix
   ```
3) （可选）签名 MSI  
   - 若有正式证书（/a 自动选取证书存储，或用 /f 指定 PFX）：
     ```powershell
     & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign `
       /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com /a `
       target\wix\Oriv-0.1.0-x86_64.msi
     ```
   - 若无证书，可自签用于测试（外部分发仍不受信任）：
     ```powershell
     $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Oriv Dev" `
       -KeyExportPolicy Exportable -CertStoreLocation Cert:\CurrentUser\My `
       -NotAfter (Get-Date).AddYears(2)
     $pwd = ConvertTo-SecureString "orivpass" -AsPlainText -Force
     Export-PfxCertificate -Cert $cert -FilePath "$env:USERPROFILE\oriv-dev.pfx" -Password $pwd
     & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign `
       /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com `
       /f "$env:USERPROFILE\oriv-dev.pfx" /p "orivpass" `
       target\wix\Oriv-0.1.0-x86_64.msi
     ```
     在要安装的机器上需手动信任证书（导入到“受信任的根证书颁发机构”和“受信任的发布者”）。

## 构建自定义引导程序（Slint）
### 单文件发布（内嵌 MSI，推荐）
```powershell
cd installer-bootstrap
$env:ORIV_MSI_PATH="../target/wix/Oriv-0.1.0-x86_64.msi"
cargo build --release --features embed_msi
```
产物：`installer-bootstrap\target\release\oriv-bootstrapper.exe`（内含 MSI）。  
如遇“拒绝访问”无法覆盖 exe，先关闭正在运行的 `oriv-bootstrapper.exe` 再重试。

### 双文件发布（不内嵌）
```powershell
cd installer-bootstrap
cargo build --release
```
将生成的 `oriv-bootstrapper.exe` 与 MSI 放同目录，或通过环境变量指定：
```powershell
$env:ORIV_MSI_PATH=".\Oriv-0.1.0-x86_64.msi"
.\oriv-bootstrapper.exe
```

### （可选）签名引导程序 exe
```powershell
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign `
  /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com /a `
  installer-bootstrap\target\release\oriv-bootstrapper.exe
```
或用自签 PFX 替换 `/a` 为 `/f PFX路径 /p 密码`。

## 运行与分发
- 单文件方案：分发签名后的 `oriv-bootstrapper.exe`，右键“以管理员身份运行”。
- 双文件方案：同目录放 `oriv-bootstrapper.exe` 与 MSI，或使用 ORIV_MSI_PATH 指向 MSI。
- 日志位置：`%TEMP%\oriv_installer\install.log`（引导界面有“打开日志”按钮）。

## UI 定制入口
- 引导界面 Slint 文件：`installer-bootstrap/ui/setup-window.slint`，已包含多页框架（欢迎/路径/进度/完成）、渐变背景、左右分栏。可在此继续美化（插画、动画、配色等），无需改后端逻辑。*** End Patch***"##
