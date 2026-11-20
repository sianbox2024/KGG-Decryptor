//! 音频转码模块：解密后的字节可以按用户选择转成 MP3 等通用格式。
//!
//! 转码通过外部 ffmpeg 进程完成（stdin/stdout 不走 Tauri IPC，避免大字节数组
//! 在前后端之间搬运）。找不到 ffmpeg 时调用方会回退为「原格式输出」，
//! 保证原有的解密功能不会因为转码组件缺失而失效。

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const DEFAULT_BITRATE: u32 = 192;
const TARGETS: &[&str] = &["mp3", "flac", "m4a", "wav", "ogg"];

#[derive(Clone, Debug)]
pub struct OutputOptions {
    /// `original` 表示保留解密后的原始编码；其余为转码目标。
    pub target: String,
    pub bitrate: u32,
}

impl OutputOptions {
    pub fn new(target: String, bitrate: u32) -> Self {
        let lower = target.to_ascii_lowercase();
        let target = if TARGETS.contains(&lower.as_str()) {
            lower
        } else {
            "original".to_string()
        };
        let bitrate = if (64..=320).contains(&bitrate) {
            bitrate
        } else {
            DEFAULT_BITRATE
        };
        Self { target, bitrate }
    }

    pub fn needs_conversion(&self, detected: &str) -> bool {
        self.target != "original" && self.target != detected.to_ascii_lowercase()
    }
}

#[derive(Serialize)]
pub struct FfmpegStatus {
    pub available: bool,
    pub path: Option<String>,
}

#[derive(Debug)]
pub struct ConversionOutcome {
    pub data: Vec<u8>,
    pub format: String,
    pub transcoded: bool,
    pub warning: Option<String>,
}

impl ConversionOutcome {
    fn passthrough(audio: &[u8], format: &str) -> Self {
        Self {
            data: audio.to_vec(),
            format: format.to_string(),
            transcoded: false,
            warning: None,
        }
    }
}

/// 前端查询转码引擎是否可用，用于在设置区给出明确提示。
pub fn ffmpeg_status() -> FfmpegStatus {
    match find_ffmpeg() {
        Some(path) => FfmpegStatus {
            available: true,
            path: Some(path.to_string_lossy().to_string()),
        },
        None => FfmpegStatus {
            available: false,
            path: None,
        },
    }
}

/// 解密后统一走这里：需要转码就转，失败则保留原格式并附带告警。
pub fn apply_output(audio: &[u8], detected: &str, opts: &OutputOptions) -> ConversionOutcome {
    if !opts.needs_conversion(detected) {
        return ConversionOutcome::passthrough(audio, detected);
    }
    match convert_with_ffmpeg(audio, opts) {
        Ok((data, format)) => ConversionOutcome {
            data,
            format,
            transcoded: true,
            warning: None,
        },
        Err(e) => {
            let mut outcome = ConversionOutcome::passthrough(audio, detected);
            outcome.warning = Some(format!("转码失败（{e}），已按原格式输出"));
            outcome
        }
    }
}

static FFMPEG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

/// 查找 ffmpeg：优先用户指定，其次应用同目录 / tools 目录，最后 PATH。
/// 结果缓存，避免每处理一个文件都启动一次 `ffmpeg -version`。
fn find_ffmpeg() -> Option<PathBuf> {
    FFMPEG_PATH.get_or_init(search_ffmpeg).clone()
}

fn search_ffmpeg() -> Option<PathBuf> {
    if let Ok(name) = std::env::var("KGG_FFMPEG") {
        let path = PathBuf::from(name);
        if works(&path) {
            return Some(path);
        }
    }

    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.to_path_buf());
            dirs.push(parent.join("tools"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }
    for dir in dirs {
        for name in ["ffmpeg.exe", "ffmpeg"] {
            let path = dir.join(name);
            if path.is_file() && works(&path) {
                return Some(path);
            }
        }
    }
    // 裸文件名交给系统 PATH 解析。Windows 下 PATH 通常包含 .exe 后缀补全。
    for name in ["ffmpeg", "ffmpeg.exe"] {
        let path = PathBuf::from(name);
        if works(&path) {
            return Some(path);
        }
    }
    None
}

fn works(path: &Path) -> bool {
    let mut cmd = Command::new(path);
    cmd.arg("-version").stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn push_encode_args(cmd: &mut Command, opts: &OutputOptions, cover: bool) {
    cmd.args(["-map", "0:a:0"]);
    if cover {
        // 封面以 attached_pic 形式保留；无封面文件时 0:v? 是可选映射，不会报错。
        cmd.args(["-map", "0:v?", "-c:v", "copy"]);
    }
    cmd.args(["-map_metadata", "0"]);
    match opts.target.as_str() {
        "mp3" => {
            cmd.args(["-c:a", "libmp3lame", "-b:a", &format!("{}k", opts.bitrate)]);
            cmd.args(["-id3v2_version", "3"]);
        }
        "m4a" => {
            cmd.args(["-c:a", "aac", "-b:a", &format!("{}k", opts.bitrate)]);
        }
        "flac" => {
            cmd.args(["-c:a", "flac"]);
        }
        "wav" => {
            cmd.args(["-c:a", "pcm_s16le"]);
        }
        "ogg" => {
            cmd.args(["-c:a", "libvorbis", "-q:a", "5"]);
        }
        _ => {}
    }
}

fn run_ffmpeg(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    opts: &OutputOptions,
    cover: bool,
) -> Result<(), String> {
    let mut cmd = Command::new(ffmpeg);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.args(["-hide_banner", "-loglevel", "error", "-y"])
        .arg("-i")
        .arg(input);
    push_encode_args(&mut cmd, opts, cover);
    cmd.arg("-f").arg(&opts.target).arg(output);

    let out = cmd
        .output()
        .map_err(|e| format!("启动 ffmpeg 失败: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let err: String = err.chars().rev().take(300).collect::<Vec<_>>().into_iter().rev().collect();
    Err(format!("ffmpeg 退出码 {:?}: {}", out.status.code(), err))
}

/// 用临时目录承载输入/输出，避免 Windows 上文件句柄占用，也避免管道死锁。
fn convert_with_ffmpeg(audio: &[u8], opts: &OutputOptions) -> Result<(Vec<u8>, String), String> {
    let ffmpeg = find_ffmpeg().ok_or("未找到 ffmpeg（可放到 exe 同目录或设置 KGG_FFMPEG）")?;
    let dir = tempfile::tempdir().map_err(|e| format!("创建临时目录失败: {e}"))?;
    let input = dir.path().join("input.bin");
    let output = dir.path().join(format!("output.{}", opts.target));
    fs::write(&input, audio).map_err(|e| format!("写入临时文件失败: {e}"))?;

    // 先尝试保留封面；少数容器的封面无法直接复制，再退回纯音频转码。
    let mut result = run_ffmpeg(&ffmpeg, &input, &output, opts, true);
    if result.is_err() {
        let _ = fs::remove_file(&output);
        result = run_ffmpeg(&ffmpeg, &input, &output, opts, false);
    }
    result?;

    let data = fs::read(&output).map_err(|e| format!("读取转码结果失败: {e}"))?;
    if data.is_empty() {
        return Err("转码结果为空".into());
    }
    Ok((data, opts.target.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_target_never_touches_audio() {
        let opts = OutputOptions::new("original".into(), 192);
        let out = apply_output(b"ID3 fake", "mp3", &opts);
        assert_eq!(out.data, b"ID3 fake");
        assert_eq!(out.format, "mp3");
        assert!(!out.transcoded);
        assert!(out.warning.is_none());
    }

    #[test]
    fn same_as_detected_target_is_passthrough() {
        let opts = OutputOptions::new("MP3".into(), 128);
        assert!(!opts.needs_conversion("mp3"));
        let out = apply_output(b"ID3 fake", "mp3", &opts);
        assert!(!out.transcoded);
    }

    #[test]
    fn invalid_options_fall_back_to_original() {
        let opts = OutputOptions::new("avi".into(), 99999);
        assert_eq!(opts.target, "original");
        assert_eq!(opts.bitrate, DEFAULT_BITRATE);
    }

    /// 生成一个 1 秒 8kHz 单声道 PCM WAV，用于验证真实转码链路。
    fn tiny_wav() -> Vec<u8> {
        let samples: Vec<u8> = (0..8000).map(|i| ((i % 200) as u8).wrapping_sub(100)).collect();
        let data_len = samples.len() as u32;
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&8000u32.to_le_bytes());
        wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        wav.extend_from_slice(&samples);
        wav
    }

    #[test]
    fn converts_wav_to_real_mp3_when_ffmpeg_available() {
        if find_ffmpeg().is_none() {
            return; // 未安装 ffmpeg 的环境跳过真实转码验证
        }
        let opts = OutputOptions::new("mp3".into(), 128);
        let out = apply_output(&tiny_wav(), "wav", &opts);
        assert!(out.transcoded, "warning: {:?}", out.warning);
        assert_eq!(out.format, "mp3");
        assert_eq!(&out.data[..3], b"ID3");
        assert!(out.warning.is_none());
    }

    #[test]
    fn mp3_output_keeps_cover_art_when_source_has_it() {
        if find_ffmpeg().is_none() || !works(Path::new("ffprobe")) {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let wav_path = dir.path().join("a.wav");
        let png_path = dir.path().join("cover.png");
        let flac_path = dir.path().join("cover.flac");
        fs::write(&wav_path, tiny_wav()).unwrap();
        let make_png = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi",
                   "-i", "color=c=red:s=64x64:d=1", "-frames:v", "1"])
            .arg(&png_path)
            .status()
            .unwrap();
        assert!(make_png.success());

        let build = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(&wav_path)
            .arg("-i")
            .arg(&png_path)
            .args(["-map", "0:a", "-map", "1:v", "-c:a", "flac", "-c:v", "png",
                   "-disposition:v:0", "attached_pic"])
            .arg(&flac_path)
            .status()
            .unwrap();
        assert!(build.success());

        let flac = fs::read(&flac_path).unwrap();
        let opts = OutputOptions::new("mp3".into(), 128);
        let out = apply_output(&flac, "flac", &opts);
        assert!(out.transcoded, "warning: {:?}", out.warning);
        let mp3_path = dir.path().join("out.mp3");
        fs::write(&mp3_path, out.data).unwrap();

        let probe = Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "stream=codec_type", "-of", "csv=p=0"])
            .arg(&mp3_path)
            .output()
            .unwrap();
        let streams = String::from_utf8_lossy(&probe.stdout);
        assert!(streams.contains("video"), "封面未保留: {streams}");
    }

    #[test]
    fn broken_input_keeps_original_with_warning() {
        if find_ffmpeg().is_none() {
            return;
        }
        let opts = OutputOptions::new("mp3".into(), 128);
        let out = apply_output(b"not audio at all", "flac", &opts);
        assert!(!out.transcoded);
        assert_eq!(out.format, "flac");
        assert_eq!(out.data, b"not audio at all");
        assert!(out.warning.is_some());
    }
}
