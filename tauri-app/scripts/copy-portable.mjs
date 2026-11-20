#!/usr/bin/env node
// 把交叉编译产出的便携版 EXE 复制到 dist/。
// 用法: node scripts/copy-portable.mjs [target-triple]
// 默认 target 为 x86_64-pc-windows-msvc，与 build:windows 脚本保持一致。

import { copyFile, mkdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = process.argv[2] ?? "x86_64-pc-windows-msvc";

const configPath = path.join(appDir, "src-tauri", "tauri.conf.json");
const config = JSON.parse((await readFile(configPath, "utf8")).replace(/^\uFEFF/, ""));

const version = config.version;
const exeName = config.mainBinaryName ?? config.productName;
const arch = target.startsWith("i686")
  ? "x86"
  : target.startsWith("aarch64")
    ? "arm64"
    : "x64";

const source = path.join(appDir, "src-tauri", "target", target, "release", `${exeName}.exe`);
const outDir = path.join(appDir, "dist");
const dest = path.join(outDir, `${exeName}_${version}_${arch}-portable.exe`);

try {
  await stat(source);
} catch {
  console.error(`未找到便携版 EXE: ${source}`);
  console.error("请先运行 npm run build:windows。");
  process.exit(1);
}

await mkdir(outDir, { recursive: true });
await copyFile(source, dest);

const { size } = await stat(dest);
console.log(
  `便携版已复制: ${path.relative(process.cwd(), dest)} (${(size / 1024 / 1024).toFixed(1)} MB)`,
);

// ffmpeg 与 EXE 放在一起，程序启动时即可发现；缺失时程序仍能解密并原格式输出。
const ffmpegName = target.includes("windows") ? "ffmpeg.exe" : "ffmpeg";
const ffmpegSrc = path.join(appDir, "src-tauri", "tools", ffmpegName);
try {
  await stat(ffmpegSrc);
  const ffmpegDest = path.join(outDir, ffmpegName);
  await copyFile(ffmpegSrc, ffmpegDest);
  console.log(`转码引擎已复制: ${path.relative(process.cwd(), ffmpegDest)}`);
  const licenseSrc = path.join(appDir, "src-tauri", "tools", "ffmpeg-LICENSE.txt");
  try {
    await stat(licenseSrc);
    const licenseDest = path.join(outDir, "ffmpeg-LICENSE.txt");
    await copyFile(licenseSrc, licenseDest);
    console.log(`许可文件已复制: ${path.relative(process.cwd(), licenseDest)}`);
  } catch {
    // 许可文件可选，但建议随 ffmpeg 一起分发。
  }
} catch {
  console.warn(
    `未找到 ${ffmpegSrc}，便携版将只能保持原格式输出。请按 src-tauri/tools/README.md 放置 ffmpeg。`,
  );
}
