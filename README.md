# KGG Decryptor

KGG Decryptor 是一个基于 Tauri 2 的本地音乐处理工具。它可以读取本机客户端已经具备播放条件的文件，将 KGG、KGE（酷狗概念版缓存）、KGM、KGMA、VPR、NCM、KWM、传统 QMC 以及旧式 MFLAC/MGG 音乐解密还原，**并可一键转码为 MP3**（也可选 FLAC / M4A / WAV / OGG 或保持原格式），最终目的是让下载的音乐能在任何支持 MP3 的设备上直接播放；同时支持 KRC 加密歌词转换（单个或批量）为标准 LRC。

> 本项目只处理你本机已经具备播放和下载条件的文件，不提供绕过会员、绕过授权或在线破解能力。

## 预览


### 主界面 — 手动解密（日间模式）


![手动解密页面](docs/screenshots/app-decrypt-light.png)

### 自动监控


![自动监控页面](docs/screenshots/app-monitor.png)

### 批量工具（批量解密 + 歌词转换）


![批量工具页面](docs/screenshots/app-batch.png)

### 歌曲库


![歌曲库页面](docs/screenshots/app-library.png)

### 夜间模式


![夜间模式](docs/screenshots/app-dark.png)

## 功能

应用采用侧边栏导航，分为四个功能页面：

### 🎵 手动解密

- 拖拽或点击选择受支持的加密音乐、普通音频或 KRC 文件，立即处理。
- 可在页面顶部设置「音频输出」目录（默认系统下载目录）；转换完成后每条记录右侧有文件夹图标，点击可在资源管理器中定位并选中生成的音频文件。
- 拖拽区域覆盖整个「手动解密」页面，不必精确拖进虚线框。
- 默认输出 MP3（192 kbps）；目标格式与码率由侧边栏「输出设置」统一控制，可选 FLAC / M4A / WAV / OGG / 保持原格式。
- 转码保留元数据与封面；转码失败时自动退回原格式并提示原因，不会丢掉解密结果。
- KRC 歌词即时转换为标准 LRC（UTF-8 编码）。
- 单文件处理进度与结果实时显示。

### 👁 自动监控

- 一键开关，监听目录，新文件下载完成后自动解密。
- 监听目录、输出目录可自定义，支持浏览选择，配置自动记忆。
- 实时动态面板展示每个文件的处理状态（检测 / 成功 / 失败）。
- 运行状态在侧边栏显示呼吸指示灯。

### 📦 批量工具

- **批量解密音乐**：选择文件夹，一次性解密其中所有受支持的音乐文件，报告成功 / 失败数量。
- **批量转换歌词**：选择 KRC 歌词目录，批量转换为标准 LRC，输出目录可自定义。
- KRC 转换不依赖数据库密钥，任何 KRC 文件（`krc1` 魔术头）均可转换。

### 📚 歌曲库

- 自动定位并解密读取酷狗本地数据库 `KGMusicV3.db`。
- 浏览已下载歌曲：歌名、音质、码率、时长、文件大小、密钥状态。

### 🌗 界面

- 日间 / 夜间双主题，一键切换，自动记住选择（默认日间）。
- 所有解密、转换、数据库读取均在本地完成，不上传任何数据。

## 输出与转码

解密和转码是两步：先按加密格式还原出原始音频（MP3 / FLAC / OGG / WAV / AAC / M4A / WMA / DFF / APE / WV / DSF 等），再由 ffmpeg 按选择的格式重新编码。默认目标为 **MP3（192 kbps）**，这是为了能直接在其他播放器、车机、手机上使用；选择「保持原格式（不转码）」即可无损保留解密结果。

- 侧边栏的「输出设置」对手动解密、自动监控、批量解密三处同时生效，并自动记忆。
- 依赖外部 **ffmpeg**，便携版与安装包已附带 LGPL 构建的 ffmpeg（Windows x64）；也可通过环境变量 `KGG_FFMPEG` 指定其他路径。详见 `tauri-app/src-tauri/tools/README.md`。
- 找不到 ffmpeg 或转码失败时，程序仍会输出解密后的原格式文件，并在界面提示，解密功能不受影响。
- MP3 是有损编码；需要无损请选 FLAC，或选「保持原格式」。歌词 KRC → LRC 的转换不经过 ffmpeg。

## 使用前提

KGG、KGM v5 和部分 KGMA 文件转换依赖酷狗客户端写入本地数据库的解密密钥，因此使用前需要满足以下条件：

1. 你是酷狗会员，且已在酷狗客户端中成功下载过目标歌曲。
2. 酷狗客户端已在本地 `KGMusicV3.db` 数据库中写入该歌曲对应的 `eKey`。
3. 本工具能读取到本机的酷狗数据库和目标 KGG 文件。

**安卓酷狗概念版**（KugouYoung）用户：`.kge` 标准音质缓存文件无需任何密钥，直接拖入即可解密；`.kgg` / `.kgg.flac` VIP 文件的密钥存放在 `/data/data/com.kugou.android.lite/files/mmkv/mggkey_multi_process`（需要 root 权限取出），把它和加密文件放在 exe 同目录即可自动识别。

简单说：酷狗客户端已经能在本机播放、且已经下载过的歌，本工具才有机会转换；数据库中没有对应密钥的文件无法解密。

KRC 歌词转换不依赖数据库密钥。KRC 文件以 `krc1` 魔术头开头，经过固定 XOR 密钥解密和 zlib 解压后，可转换为 UTF-8 编码的 LRC 文本（逐行时间标签已转换为标准 `[mm:ss.xx]` 格式）。

## 支持格式

| 后缀 | 来源 | 密钥要求 |
|---|---|---|
| `.kgg` | 酷狗 Hi-Res | 本机 `KGMusicV3.db`，或安卓概念版 `mggkey_multi_process` |
| `.kge` | 酷狗概念版缓存 | 无需密钥（KGM v3 固定算法） |
| `.kgm` / `.kgma` / `.vpr` | 酷狗 | v5 版本需要 `KGMusicV3.db`，v3 可离线处理 |
| `.ncm` | 网易云音乐 | 文件内置密钥 |
| `.kwm` | 酷我音乐 | 文件内置密钥 |
| `.qmc0/.qmc2/.qmc3/.qmc4/.qmc6/.qmc8` | QQ 音乐传统 QMC | 文件内置密钥或静态掩码 |
| `.qmcflac/.qmcogg/.tkm` | QQ 音乐传统 QMC | 文件内置密钥或静态掩码 |
| `.mflac/.mflac0/.mflac1` / `.mgg/.mgg0/.mgg1/.mggl` | QQ 音乐 | 旧式尾部 ekey 可离线处理；新版 `musicex` 暂不联网取钥 |
| `.krc` | 酷狗歌词 | 固定 XOR + zlib |
| `.mp3/.flac/.ogg/.wav/.m4a/.aac/.ape/.wv/.dff/.dsf` | 未加密音频 | 无需密钥，直接透传或转码 |

酷狗自家的加密容器为 `.kgg` / `.kge` / `.kgm` / `.kgma` / `.vpr`。分发不只依赖后缀：未知后缀、无后缀和安卓客户端写入的 `.kgg.flac` 双后缀都会按文件头识别。
## 工作原理

所有加密格式由 `formats.rs` 统一分发：扩展名只作辅助，优先按文件头魔术字节识别容器；解密后再次根据音频文件头判断真实编码。随后 `transcode.rs` 按「输出设置」决定是否交给 ffmpeg 转码（默认转 MP3）。

以依赖本地数据库的酷狗 KGG / KGM v5 为例：

```text
酷狗客户端下载歌曲
       |
       v
KGG / KGM v5 加密文件 + KGMusicV3.db（含 eKey 密钥）
       |
       v
读取并解密本地数据库 -> 提取歌曲 eKey
       |
       v
eKey 经 TEA + Base64 多层推导 -> 得到原始密钥
       |
       v
根据密钥类型选择 MapCipher / RC4Cipher -> 解密音频数据
       |
       v
根据文件头识别格式 -> 输出 MP3 / FLAC / OGG / WAV 等
```

安卓酷狗概念版（KugouYoung）的密钥来源不同：

```text
概念版客户端下载歌曲（在线从 tracker.kugou.com/v6/get_ekey 获取 ekey）
       |
       v
ekey 写入本机 MMKV 文件 mggkey_multi_process（hash -> ekey 键值对）
       |
       v
root 后取出该文件，放 exe 同目录
       |
       v
.kge 缓存文件无需密钥直接解密（KGM v3 固定算法）
.kgg / .kgg.flac 文件按 audio_hash 查 MMKV 中的 ekey 解密
```

不依赖数据库的格式则直接解密：KGM v3 / VPR 使用内置 XOR 链，NCM / KWM 使用文件内置密钥，传统 QMC 与旧式 MFLAC / MGG 使用静态掩码或尾部 ekey（与 unlock-music 算法兼容，musicex 新版明确拒绝离线取钥）。

KRC 歌词转换流程：

```text
KRC 文件（krc1 魔术头）
       |
       v
固定 XOR 密钥逐字节异或 -> zlib 解压
       |
       v
解析原始歌词 JSON -> 逐行转换为 [mm:ss.xx] 时间标签
       |
       v
输出 UTF-8 编码的标准 LRC 文件
```

关键技术点：

- KGG 文件头包含 KGM Magic、音频偏移量和音频哈希。
- `KGMusicV3.db` 数据库经过 AES-128-CBC 分页加密，需要先解密再查询。
- `eKey` 经过 TEA 和 Base64 多层封装，需要推导出原始密钥。
- 解密算法根据密钥长度选择 MapCipher 或 RC4Cipher。
- 输出格式通过音频文件头魔术字节自动判断，例如 `ID3`、`fLaC`、`OggS`。

## 技术栈

- Tauri 2：桌面应用框架。
- Rust：后端解密、文件监听、数据库读取和批量转换逻辑。
- Vanilla HTML/CSS/JavaScript：前端单页界面（无框架依赖）。
- aes / cbc / md-5：解密酷狗本地数据库。
- rusqlite：读取解密后的 SQLite 数据库。
- notify：监听文件系统变更。
- flate2：解压 KRC 歌词内容。
- ffmpeg（外部进程）：按选择的目标格式重新编码音频；随包分发 LGPL 构建。

## 项目结构

```text
.
├── README.md                    # 项目说明和使用文档
├── dev.bat                      # Windows 开发模式启动脚本
├── LICENSE
├── docs/
│   └── screenshots/             # 界面截图（README 引用）
└── tauri-app/
    ├── package.json            # Tauri CLI 脚本和前端侧依赖
    ├── package-lock.json
    ├── src/
    │   └── index.html          # 桌面端单页界面
    └── src-tauri/
        ├── Cargo.toml          # Rust 依赖和构建配置
        ├── tauri.conf.json     # Tauri 应用配置
        ├── build.rs
        ├── capabilities/
        │   └── default.json    # Tauri 权限配置
        ├── icons/              # 应用图标
        ├── tools/              # 随包分发的 ffmpeg 放置目录
        └── src/
            ├── lib.rs          # 数据库、KRC、监听和 Tauri 命令
            ├── formats.rs      # KGG/KGM/NCM/KWM/QMC/MFLAC/MGG 统一解码器
            ├── transcode.rs    # ffmpeg 转码与输出设置
            └── main.rs         # 应用入口
```

项目中的 `node_modules/`、`dist/`、`src-tauri/target/`、本地数据库和音频文件都属于本地依赖或构建/个人数据，已通过 `.gitignore` 排除。

## 开发

### 环境要求

- Node.js 18+
- Rust stable
- Windows C/C++ 编译环境（MSVC 或 MinGW）

### 安装依赖

```bash
cd tauri-app
npm install
```

### 开发模式运行

Windows 下可直接双击项目根目录的 `dev.bat`（自动处理中文路径和自定义 Rust 安装路径的情况）。

或手动运行：

```bash
cd tauri-app
npm run dev
```

> 注意：如果你的项目路径包含中文或全角字符，且使用 MinGW (GNU) 工具链，链接器可能报错。此时请在纯英文路径下构建，或使用 MSVC 工具链。

### 构建安装包

```bash
cd tauri-app
npm run build
```

构建产物位于 `tauri-app/src-tauri/target/release/`，安装包由 Tauri 输出到对应的 bundle 目录。

`release-assets/` 存放各版本的便携版 EXE 与安装包（本地目录，不进 Git）：v1.2.0 起包含 KGM/KGMA/VPR、NCM、KWM、QMC 系列与旧式 MFLAC/MGG 支持。

## 使用方法

1. 在侧边栏「输出设置」确认目标格式（默认 MP3，192 kbps）；便携版已附带 ffmpeg.exe，无需额外安装。
2. 确认客户端已在本机下载目标歌曲（仅 KGG / KGM v5 / 部分 KGMA 需要数据库密钥，其余格式开箱即用）。
3. **手动解密**：确认「音频输出」目录（默认系统下载目录），拖拽受支持的加密音乐或 KRC 文件到窗口中，转换后点击条目右侧的文件夹图标即可打开所在目录。
4. **自动监控**：设置监听目录（客户端下载目录）和输出目录，开启后新文件自动处理。
5. **批量工具**：批量解密选择加密音乐所在文件夹；歌词转换选择 KRC 目录和 LRC 输出目录。
6. **歌曲库**：查看本地数据库中已下载歌曲的音质与密钥状态。

## 已知限制

- 当前主要面向 Windows 环境。
- MP3/M4A 是有损编码；若追求音质请选择 FLAC 或「保持原格式」。
- 转码依赖随包的 ffmpeg；缺失或转码失败时仍会输出原格式文件并提示。
- 新版 QQ MusicEx MFLAC/MGG 需要 QQ 音乐会话取钥，当前版本明确拒绝此类联网取钥。
- KGG 和 KGM v5 转换必须依赖本地数据库中的对应密钥（桌面端 `KGMusicV3.db` 或安卓概念版 `mggkey_multi_process`）；KRC 歌词转换无此限制。
- 如果酷狗更新加密方案，相关解析逻辑可能需要同步调整。

## License

MIT
