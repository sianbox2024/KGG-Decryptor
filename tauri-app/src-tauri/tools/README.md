# 转码引擎目录

本程序通过外部 ffmpeg 把解密后的音频转成 MP3。**当前仓库的 `ffmpeg.exe` 已随便携版打包**，来源与校验信息：

- 版本：`FFmpeg version N-129130-gcd39c36377-20261119`（BtbN/FFmpeg-Builds `latest` 发布，LGPL v3 静态构建，含 libmp3lame）
- 来源：https://github.com/FFmpeg-Builds/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl.zip
- ZIP SHA-256：`e390e653c931a30a5051c0fe7f6859a2d3162171566c1e422c1f97b157de0da5`
- `ffmpeg-LICENSE.txt` 为该构建的 LGPL v3 许可证文本，随包分发。

如需替换，请把 **LGPL 构建的 ffmpeg** 放在本目录：

- Windows：`ffmpeg.exe`
- 其他平台：`ffmpeg`

程序启动时会按以下顺序查找：环境变量 `KGG_FFMPEG` → 可执行文件同目录 → 可执行文件同目录的 `tools/` → 当前工作目录 → 系统 PATH。
`npm run build:windows` 会把本目录中的 ffmpeg 一起复制进 `dist/` 便携版；`bundle.resources` 会把它打进 NSIS / MSI 安装包。

获取方式（任选其一）：

- 从 https://www.gyan.dev/ffmpeg/builds/ 下载 `ffmpeg-release-essentials`（LGPL 构建），解压后取 `bin/ffmpeg.exe`。
- 从 https://ffmpeg.org/download.html 选择对应平台的 LGPL 构建。

> 许可提醒：请使用 LGPL 构建的 ffmpeg。GPL 构建会引入额外的开源源码义务；
> 若使用 GPL 构建，需要自行履行对应许可要求。第三方声明见项目根目录
> `THIRD-PARTY-NOTICES.md`。
