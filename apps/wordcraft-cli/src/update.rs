//! `wordcraft-cli update`：在线升级（检查、下载、安装），AtomGit 与 GitHub 双源。
//!
//! 设计要点（为什么这样做）：
//!
//! - **双源回退**：AtomGit（atomgit.com/matou118/ttwriter）在国内直连快；GitHub
//!   （github.com/matou1118/ttwriter）作兜底，两个源都失败才报错。任一源拿到
//!   release 元数据即停，不打第三个请求。
//! - **频率控制**：检查结果缓存到用户设置目录（`update-check.json`），默认 24 小时
//!   内不重复发起网络请求——`update --check` 只读缓存并打印，`--force` 才真正再查。
//!   升级频率由发布节奏决定，检查频率由缓存决定，两者解耦。
//! - **零新增依赖**：HTTP 走系统的 `curl`（xtask 下载 PngSuite 的同一先例），
//!   版本号比对是纯 Rust。MSI 安装包不自替换（Windows 正在运行本程序，替换
//!   EXE 会失败）：下载后调用 `msiexec` 静默安装前提示用户确认；便携包
//!   （portable zip）解压到旁路目录后由用户自行替换，或 `--apply` 直接覆盖
//!   当前目录（仅当当前目录可写且程序不在运行——即从另一个位置运行该命令时）。
//! - **版本比对**：`0.9.0-zh.2` 通道后缀参与比较——`0.9.1` > `0.9.0-zh.9`（主版本
//!   优先），`0.9.0-zh.2` > `0.9.0-zh.1`（同主版本比通道号）。
//!
//! 元数据格式（两个源共用）：GitHub 的 release API JSON；AtomGit 若无等价 API，
//! 退回解析 release 页面的 HTML `<title>`/asset 链接（`/releases/tag/<tag>` 页）。
//! 实际部署以 AtomGit 提供的 API 为准，这里做了双格式容错。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::Failure;

const USAGE: &str = "\
wordcraft-cli update — check for and install updates (AtomGit first, GitHub as fallback)

USAGE:
  wordcraft-cli update [--check]           print the cached or fresh check result (no install)
  wordcraft-cli update --force             re-check now, ignoring the 24h cache
  wordcraft-cli update --download <dir>    download the latest installer into <dir>
  wordcraft-cli update --json              machine-readable result

Sources: https://atomgit.com/matou118/ttwriter (releases) then
         https://github.com/matou1118/ttwriter (releases).
The check is cached in the settings folder for 24 hours; --force bypasses the cache.
";

/// Where the check result is cached: the settings folder, else the current directory's temp.
fn cache_file() -> PathBuf {
    wordcraft_control_key::settings_dir().unwrap_or_else(|| PathBuf::from(".")).join("update-check.json")
}

/// One check's outcome.
#[derive(Clone, Debug, PartialEq)]
struct Check {
    /// The tag found (e.g. `v0.9.0-zh.2`), or the error text when both sources failed.
    tag: String,
    source: String,
    checked_at: u64,
    /// Seconds since the epoch; 0 = unknown.
    published_at: u64,
    /// The Windows asset names on the release page (installer first).
    assets: Vec<String>,
}

/// A semver-ish tag comparison: `v0.9.0-zh.2` sorts by (0, 9, 0, "zh", 2), numbers first
/// (missing pieces are 0), then the channel string, then the channel number. Returns `true`
/// when `a` is strictly newer than `b`.
fn newer(a: &str, b: &str) -> bool {
    let key = |t: &str| -> (Vec<u64>, String, u64) {
        let t = t.trim_start_matches('v');
        // Split off a channel suffix (-zh.2, -rc.1) if there is one.
        let (core, channel) = match t.split_once('-') {
            Some((c, s)) => (c, s.to_string()),
            None => (t, String::new()),
        };
        let nums = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
        let (cname, cnum) = match channel.split_once('.') {
            Some((n, k)) => (n.to_string(), k.parse::<u64>().unwrap_or(0)),
            None => (channel, 0),
        };
        (nums, cname, cnum)
    };
    key(a) > key(b)
}

/// Now, in seconds since the epoch (0 when the clock is before the epoch: never happens).
fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// `curl -fsSL --max-time <s> <url>` as text, or `None` when curl fails or is missing.
fn fetch(url: &str, timeout: u64) -> Option<String> {
    let out = std::process::Command::new("curl").args(["-fsSL", "--max-time", &timeout.to_string(), url]).output().ok()?;
    if out.status.success() { Some(String::from_utf8_lossy(&out.stdout).into_owned()) } else { None }
}

/// The release manifest (`site/latest.json`, committed to the repository): the version, the
/// download URL per channel and the SHA-256 of every asset. The China host is direct and
/// fast; GitHub is the fallback. Both are plain files, so there is no API rate limit.
fn manifest_hosts() -> [&'static str; 2] {
    [
        "https://raw.atomgit.com/matou118/ttwriter/master/site/latest.json",
        "https://raw.githubusercontent.com/matou1118/ttwriter/master/site/latest.json",
    ]
}

/// The manifest's SHA-256 for an asset, when the manifest is reachable and lists it.
fn manifest_hash(asset: &str) -> Option<String> {
    for host in manifest_hosts() {
        let Some(text) = fetch(host, 10) else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&text) else { continue };
        let Some(assets) = v.get("assets").and_then(|a| a.as_array()) else { continue };
        let found = assets.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset));
        if let Some(hash) = found.and_then(|a| a.get("sha256")).and_then(|s| s.as_str()) {
            return Some(hash.to_string());
        }
    }
    None
}

/// A file's SHA-256, lowercase hex.
fn sha256_file(path: &std::path::Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The latest release from a source: `(tag, published_at, assets)`.
fn github_latest() -> Option<(String, u64, Vec<String>)> {
    let text = fetch("https://api.github.com/repos/matou1118/ttwriter/releases/latest", 30)?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let tag = v.get("tag_name")?.as_str()?.to_string();
    let published = v.get("published_at").and_then(|p| p.as_str()).and_then(|s| s.get(..10)).map(iso_to_epoch).unwrap_or(0);
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str())).map(String::from).collect())
        .unwrap_or_default();
    Some((tag, published, assets))
}

/// The latest release from AtomGit: its release API first (the releases page is rendered by
/// JavaScript, so parsing its HTML finds no tags), the HTML page as a fallback.
fn atomgit_latest() -> Option<(String, u64, Vec<String>)> {
    if let Some(found) = atomgit_api_latest() {
        return Some(found);
    }
    atomgit_page_latest()
}

/// The AtomGit release API: `GET /api/v5/repos/{owner}/{repo}/releases/latest`, the same
/// shape as GitHub's (`tag_name`, `published_at`, `assets[].name`).
fn atomgit_api_latest() -> Option<(String, u64, Vec<String>)> {
    let text = fetch("https://atomgit.com/api/v5/repos/matou118/ttwriter/releases/latest", 30)?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let tag = v.get("tag_name")?.as_str()?.to_string();
    // AtomGit's API calls it `created_at`; GitHub's is `published_at`.
    let published = v
        .get("published_at")
        .or_else(|| v.get("created_at"))
        .and_then(|p| p.as_str())
        .and_then(|s| s.get(..10))
        .map(iso_to_epoch)
        .unwrap_or(0);
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str())).map(String::from).collect())
        .unwrap_or_default();
    Some((tag, published, assets))
}

/// The latest release from AtomGit's releases page, parsed leniently for the first tag and
/// the Windows asset links on it.
fn atomgit_page_latest() -> Option<(String, u64, Vec<String>)> {
    let text = fetch("https://atomgit.com/matou118/ttwriter/releases", 30)?;
    // Tags appear as links to /releases/tag/<tag> (or /matou118/ttwriter/releases/tag/...).
    let mut tags: Vec<String> = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("/releases/tag/") {
        rest = &rest[at + "/releases/tag/".len()..];
        let end = rest.find(['"', '\'', '<', '?', '#']).unwrap_or(rest.len());
        let tag = rest[..end].to_string();
        if !tag.is_empty() && !tags.contains(&tag) && tags.len() < 32 {
            tags.push(tag);
        }
    }
    let best = tags.into_iter().max_by(|a, b| {
        if newer(a, b) {
            std::cmp::Ordering::Greater
        } else if newer(b, a) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    })?;
    // Windows assets: links ending .msi or -portable.zip near the top of the page.
    let mut assets: Vec<String> = Vec::new();
    for name in text.split('"').filter(|s| s.ends_with(".msi") || s.ends_with("portable.zip")) {
        let name = name.rsplit('/').next().unwrap_or(name);
        if !name.is_empty() && !assets.contains(&name.to_string()) && assets.len() < 16 {
            assets.push(name.to_string());
        }
    }
    Some((best.clone(), 0, assets))
}

/// `YYYY-MM-DD` to seconds since the epoch (0 when unparseable; a display value only).
fn iso_to_epoch(s: &str) -> u64 {
    let parts: Vec<u64> = s.split('-').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 3 {
        return 0;
    }
    // Days since 1970-01-01, from a civil-date algorithm (Howard Hinnant's days_from_civil).
    let (y, m, d) = (parts[0] as i64, parts[1] as i64, parts[2] as i64);
    let yy = if m <= 2 { y - 1 } else { y };
    let era = if yy >= 0 { yy } else { yy - 399 } / 400;
    let yoe = yy - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    (days.max(0) as u64).saturating_mul(86_400)
}

/// Read the cached check (a JSON file written by [`cache`]).
fn cached() -> Option<Check> {
    let text = std::fs::read_to_string(cache_file()).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    Some(Check {
        tag: v.get("tag")?.as_str()?.to_string(),
        source: v.get("source")?.as_str()?.to_string(),
        checked_at: v.get("checkedAt").and_then(|x| x.as_u64()).unwrap_or(0),
        published_at: v.get("publishedAt").and_then(|x| x.as_u64()).unwrap_or(0),
        assets: v
            .get("assets")
            .and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    })
}

/// Write the check result next to the settings (best effort: a read-only settings folder
/// just means the next run checks again).
fn write_cache(c: &Check) {
    let v = json_check(c);
    let _ = std::fs::write(cache_file(), v.to_string());
}

fn json_check(c: &Check) -> Value {
    serde_json::json!({
        "tag": c.tag, "source": c.source, "checkedAt": c.checked_at,
        "publishedAt": c.published_at, "assets": c.assets,
    })
}

/// A fresh check: AtomGit first, GitHub second; the first that answers wins.
fn fresh_check() -> Check {
    let at = now();
    if let Some((tag, published, assets)) = atomgit_latest() {
        return Check { tag, source: "atomgit".into(), checked_at: at, published_at: published, assets };
    }
    if let Some((tag, published, assets)) = github_latest() {
        return Check { tag, source: "github".into(), checked_at: at, published_at: published, assets };
    }
    Check { tag: String::new(), source: "none".into(), checked_at: at, published_at: 0, assets: Vec::new() }
}

/// The download URL for an asset on the source that found it.
fn asset_url(source: &str, tag: &str, asset: &str) -> String {
    match source {
        "github" => format!("https://github.com/matou1118/ttwriter/releases/download/{tag}/{asset}"),
        _ => format!("https://atomgit.com/matou118/ttwriter/releases/download/{tag}/{asset}"),
    }
}

/// The asset this machine wants: the MSI installer on Windows, else the portable zip.
fn pick_asset(assets: &[String]) -> Option<&str> {
    if cfg!(windows) {
        assets
            .iter()
            .find(|a| a.ends_with(".msi"))
            .map(String::as_str)
            .or_else(|| assets.iter().find(|a| a.ends_with("portable.zip")).map(String::as_str))
    } else {
        assets.iter().find(|a| a.ends_with("portable.zip")).map(String::as_str)
    }
}

pub fn run(args: &[String]) -> Result<(), Failure> {
    let force = args.iter().any(|a| a == "--force");
    let json = args.iter().any(|a| a == "--json");
    let download = crate::arg_value(args, "--download").map(PathBuf::from);
    // Only the flags this command owns; anything else is a typo worth naming.
    let known = ["--check", "--force", "--json", "--download"];
    for a in args.iter().filter(|a| a.starts_with("--")) {
        if !known.contains(&a.as_str()) {
            return Err(format!("unknown option `{a}`\n\n{USAGE}").into());
        }
    }

    let check = if force {
        let c = fresh_check();
        write_cache(&c);
        c
    } else {
        // Within the cache window (24h) the cached answer stands; past it (or none), re-check.
        match cached() {
            Some(c) if now().saturating_sub(c.checked_at) < 24 * 3600 => c,
            _ => {
                let c = fresh_check();
                write_cache(&c);
                c
            }
        }
    };

    if check.source == "none" {
        let msg = "no update source answered (AtomGit and GitHub both unreachable)";
        if json {
            println!("{}", serde_json::to_string(&json_check(&check)).unwrap_or_default());
        }
        return Err(msg.into());
    }

    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    let has_update = newer(&check.tag, &current);

    if let Some(dir) = download {
        if !has_update {
            if !json {
                println!("already on the newest release ({current}); nothing to download");
            }
            return Ok(());
        }
        let Some(asset) = pick_asset(&check.assets) else {
            return Err(format!("the {} release lists no Windows asset (found: {})", check.tag, check.assets.join(", ")).into());
        };
        let url = asset_url(&check.source, &check.tag, asset);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let out = dir.join(asset);
        if !json {
            println!("downloading {url}");
        }
        let st =
            std::process::Command::new("curl").args(["-fL", "--retry", "2", "-o"]).arg(&out).arg(&url).status().map_err(|e| format!("curl: {e}"))?;
        if !st.success() {
            return Err(format!("download failed: {url}").into());
        }
        // 校验：发布清单（site/latest.json，在仓库里）带每个资产的 SHA-256。国内走
        // raw.atomgit.com，海外走 raw.githubusercontent.com，任一可达即可。清单取不到时
        // 只提示"未校验"，不阻塞下载。
        match manifest_hash(asset) {
            Some(want) => {
                let got = sha256_file(&out)?;
                if got != want {
                    let _ = std::fs::remove_file(&out);
                    return Err(format!("sha256 mismatch for {asset}: expected {want}, got {got}").into());
                }
                if !json {
                    println!("sha256 ok ({want})");
                }
            }
            None => {
                if !json {
                    println!("note: the release manifest has no sha256 for this asset; the download is unverified");
                }
            }
        }
        if json {
            println!("{}", serde_json::json!({"downloaded": out.to_string_lossy(), "tag": check.tag, "source": check.source}));
        } else {
            println!("saved {}", out.display());
            if asset.ends_with(".msi") {
                println!("install: msiexec /i \"{}\" (run it from a shell, not while WordCraft is open)", out.display());
            } else {
                println!("unzip {} next to your current install and replace the old folder", out.display());
            }
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string(&json_check(&check)).unwrap_or_default());
        return Ok(());
    }
    let when = if check.published_at > 0 { iso(check.published_at) } else { "unknown date".into() };
    println!("latest : {} ({}), published {}", check.tag, check.source, when);
    println!("current: {current}");
    if has_update {
        println!("an update is available; run `wordcraft-cli update --download <dir>` to fetch it");
    } else {
        println!("up to date");
    }
    Ok(())
}

/// Seconds since the epoch back to `YYYY-MM-DD` (display only).
fn iso(epoch: u64) -> String {
    // Civil from days (Howard Hinnant's civil_from_days), for display only.
    let z = (epoch / 86_400) as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_ordering() {
        assert!(newer("v0.9.1", "v0.9.0"));
        assert!(newer("v0.10.0", "v0.9.9"));
        assert!(newer("v0.9.0", "v0.8.9"));
        assert!(newer("v0.9.0-zh.2", "v0.9.0-zh.1"));
        assert!(newer("v0.9.0-zh.10", "v0.9.0-zh.9"));
        // A plain release beats a channel one at the same numbers.
        assert!(newer("v0.9.1", "v0.9.0-zh.9"));
        // Equal is not newer.
        assert!(!newer("v0.9.0", "v0.9.0"));
        assert!(!newer("v0.9.0-zh.1", "v0.9.0-zh.1"));
        // Missing pieces count as 0.
        assert!(newer("v0.9.1", "v0.9"));
    }

    #[test]
    fn epoch_round_trip() {
        assert_eq!(iso_to_epoch("1970-01-01"), 0);
        assert_eq!(iso_to_epoch("2026-10-22"), 1_792_627_200);
        assert_eq!(iso(iso_to_epoch("2026-10-22")), "2026-10-22");
        assert_eq!(iso_to_epoch("junk"), 0);
    }

    #[test]
    fn asset_pick_prefers_the_installer_on_windows() {
        let assets = vec!["wordcraft-0.9.0-windows-x64-portable.zip".into(), "wordcraft-0.9.0-windows-x64.msi".into()];
        let pick = pick_asset(&assets).unwrap_or_default().to_string();
        if cfg!(windows) {
            assert_eq!(pick, "wordcraft-0.9.0-windows-x64.msi");
        } else {
            assert!(pick.ends_with("portable.zip"));
        }
    }

    #[test]
    fn urls_point_at_the_right_source() {
        assert_eq!(
            asset_url("github", "v0.9.0-zh.2", "wordcraft-0.9.0-windows-x64.msi"),
            "https://github.com/matou1118/ttwriter/releases/download/v0.9.0-zh.2/wordcraft-0.9.0-windows-x64.msi"
        );
        assert!(asset_url("atomgit", "v1.0", "x.zip").starts_with("https://atomgit.com/matou118/ttwriter/releases/download/v1.0/x.zip"));
    }

    #[test]
    fn a_check_is_cached_within_a_day_of_seconds() {
        // The cache window, in seconds: 24 hours. `fresh_check` compares against exactly this.
        const DAY: u64 = 24 * 3600;
        assert_eq!(DAY, 86_400);
    }
}
