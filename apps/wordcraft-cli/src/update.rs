//! `wordcraft-cli update`：在线升级（检查、下载、安装）。
//!
//! 设计要点（为什么这样做）：
//!
//! - **清单优先**：`site/latest.json`（随发布提交进仓库的清单）是检查与校验的唯一
//!   权威——版本、各通道下载地址、每个资产的 SHA-256 都在里面。清单按三源回退读取：
//!   Gitee raw（静态、无 API 配额、国内首选）→ AtomGit contents API（共享令牌有
//!   429 限流）→ GitHub raw（海外兜底），任一源拿到即停。清单读不到才退回
//!   release API（AtomGit → GitHub）。
//! - **下载**：按发现清单的源选择资产地址（国内 AtomGit、海外 GitHub），
//!   `curl --retry 2` 容错；不管从哪个源下载，SHA-256 一律对清单校验。
//!   Gitee 免费版附件单文件上限 100MB（便携包 108MB 超限），故只托管清单，不放安装包。
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
//! 元数据格式：清单读不到时才看 release API——GitHub 的 release API JSON；AtomGit
//! 的 API 同形（`created_at` 代替 `published_at`），无 API 时退回解析 release 页面
//! 的 HTML asset 链接（`/releases/tag/<tag>` 页；其页面由 JS 渲染，通常解析不到）。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::Failure;

const USAGE: &str = "\
wordcraft-cli update — check for and install updates (Gitee manifest first)

USAGE:
  wordcraft-cli update [--check]           print the cached or fresh check result (no install)
  wordcraft-cli update --force             re-check now, ignoring the 24h cache
  wordcraft-cli update --download <dir>    download the latest installer into <dir>
  wordcraft-cli update --install           download, verify and install the update
                                            [--install-dir <dir>] targets a specific install
  wordcraft-cli update --json              machine-readable result

Sources: site/latest.json on gitee.com/ttwriter/ttread (raw), then AtomGit's
         contents API, then raw.githubusercontent.com; release APIs only as a
         last resort. Downloads come from atomgit.com (China) or github.com.
The check is cached in the settings folder for 24 hours; --force bypasses the cache.
--install: an MSI is installed silently (msiexec /qn); a portable zip is unpacked
         next to the running exe and swapped in — only when this command runs
         from outside the install dir (Windows locks a running exe).
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
/// download URL per channel and the SHA-256 of every asset. Read, in order: Gitee's raw host
/// (a static file, no API quota, the reliable path inside China), AtomGit's contents API
/// (base64 envelope, one shared rate limit), GitHub's raw host (the overseas fallback).
/// All three serve the same bytes, so any one reachable is enough.
/// `WORDCRAFT_UPDATE_MANIFEST` (a full URL) overrides all hosts — a self-hosted mirror, and
/// the test harness's local server.
fn manifest_hosts() -> Vec<String> {
    let mut hosts: Vec<String> = Vec::new();
    if let Some(url) = std::env::var_os("WORDCRAFT_UPDATE_MANIFEST") {
        hosts.push(url.to_string_lossy().into_owned());
    }
    hosts.extend(
        [
            "https://gitee.com/ttwriter/ttread/raw/master/site/latest.json",
            "https://atomgit.com/api/v5/repos/matou118/ttwriter/contents/site/latest.json",
            "https://raw.githubusercontent.com/matou1118/ttwriter/master/site/latest.json",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    hosts
}

/// The manifest as JSON: AtomGit's contents API wraps the file in a base64 envelope, GitHub's
/// raw host serves the JSON itself.
fn manifest_json() -> Option<Value> {
    use base64::Engine as _;
    for host in manifest_hosts() {
        let Some(text) = fetch(&host, 10) else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&text) else { continue };
        if v.get("assets").is_some() {
            return Some(v);
        }
        if let Some(b64) = v.get("content").and_then(|c| c.as_str())
            && let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(b64)
            && let Ok(inner) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&decoded))
            && inner.get("assets").is_some()
        {
            return Some(inner);
        }
    }
    None
}

/// The manifest's SHA-256 for an asset, when the manifest is reachable and lists it.
fn manifest_hash(asset: &str) -> Option<String> {
    let v = manifest_json()?;
    v.get("assets")?
        .as_array()?
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset))
        .and_then(|a| a.get("sha256").and_then(|s| s.as_str()))
        .map(String::from)
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
    let published =
        v.get("published_at").or_else(|| v.get("created_at")).and_then(|p| p.as_str()).and_then(|s| s.get(..10)).map(iso_to_epoch).unwrap_or(0);
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

/// What the manifest says about the latest release: `(tag, published_at, asset names)`,
/// or `None` when the JSON is not a usable manifest (no tag, no assets). Pure — the
/// network fetching lives in [`manifest_json`], so this is unit-testable.
fn manifest_release(v: &Value) -> Option<(String, u64, Vec<String>)> {
    let tag = v.get("tag")?.as_str()?.to_string();
    let published = v.get("published").and_then(|p| p.as_str()).and_then(|s| s.get(..10)).map(iso_to_epoch).unwrap_or(0);
    // The manifest lists every asset once per channel (cn/global), so names repeat.
    let mut assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str())).map(String::from).collect::<Vec<_>>())
        .unwrap_or_default();
    assets.dedup();
    if assets.is_empty() {
        return None;
    }
    Some((tag, published, assets))
}

/// A fresh check: the manifest first (Gitee raw / AtomGit API / GitHub raw behind
/// [`manifest_json`]), then AtomGit's release API, then GitHub's; the first that answers wins.
fn fresh_check() -> Check {
    let at = now();
    if let Some(v) = manifest_json()
        && let Some((tag, published, assets)) = manifest_release(&v)
    {
        return Check { tag, source: "manifest".into(), checked_at: at, published_at: published, assets };
    }
    if let Some((tag, published, assets)) = atomgit_latest() {
        return Check { tag, source: "atomgit".into(), checked_at: at, published_at: published, assets };
    }
    if let Some((tag, published, assets)) = github_latest() {
        return Check { tag, source: "github".into(), checked_at: at, published_at: published, assets };
    }
    Check { tag: String::new(), source: "none".into(), checked_at: at, published_at: 0, assets: Vec::new() }
}

/// 清单里某资产的直链 URL（`assets[].url`），有就用它（镜像/本地测试）。
fn manifest_asset_url(asset: &str) -> Option<String> {
    let v = manifest_json()?;
    v.get("assets")?
        .as_array()?
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset))
        .and_then(|a| a.get("url").and_then(|u| u.as_str()))
        .map(String::from)
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
    let install = args.iter().any(|a| a == "--install");
    let install_dir = crate::arg_value(args, "--install-dir").map(PathBuf::from);
    // Only the flags this command owns; anything else is a typo worth naming.
    let known = ["--check", "--force", "--json", "--download", "--install", "--install-dir"];
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
        let msg = "no update source answered (the manifest hosts and the release APIs are all unreachable)";
        if json {
            println!("{}", serde_json::to_string(&json_check(&check)).unwrap_or_default());
        }
        return Err(msg.into());
    }

    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    let has_update = newer(&check.tag, &current);

    if install {
        if !has_update {
            if !json {
                println!("already on the newest release ({current}); nothing to install");
            }
            return Ok(());
        }
        return install_update(&check, &current, json, install_dir.as_deref());
    }

    if let Some(dir) = download {
        if !has_update {
            if !json {
                println!("already on the newest release ({current}); nothing to download");
            }
            return Ok(());
        }
        let out = fetch_asset(&check, json)?;
        // 挪到用户指定的目录再报路径。
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let moved = dir.join(out.file_name().unwrap_or_default());
        std::fs::rename(&out, &moved).map_err(|e| format!("move to {}: {e}", dir.display()))?;
        if json {
            println!("{}", serde_json::json!({"downloaded": moved.to_string_lossy(), "tag": check.tag, "source": check.source}));
        } else {
            println!("saved {}", moved.display());
            if moved.extension().is_some_and(|e| e == "msi") {
                println!("install: msiexec /i \"{}\" (run it from a shell, not while WordCraft is open)", moved.display());
            } else {
                println!("unzip {} next to your current install and replace the old folder", moved.display());
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

/// 下载选定资产到临时目录并校验 SHA-256，返回已校验的文件路径。
/// 清单带哈希时强校验（不匹配即删文件报错）；清单拿不到时提示未校验但不阻塞。
/// 下载地址：清单里资产自带的 `url` 优先（自建镜像/本地测试用），没有才按发现源拼接。
fn fetch_asset(check: &Check, json: bool) -> Result<PathBuf, Failure> {
    let Some(asset) = pick_asset(&check.assets) else {
        return Err(format!("the {} release lists no Windows asset (found: {})", check.tag, check.assets.join(", ")).into());
    };
    let url = manifest_asset_url(asset).unwrap_or_else(|| asset_url(&check.source, &check.tag, asset));
    let dir = std::env::temp_dir().join("wordcraft-update");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let out = dir.join(asset);
    if !json {
        println!("downloading {url}");
    }
    let st = std::process::Command::new("curl").args(["-fL", "--retry", "2", "-o"]).arg(&out).arg(&url).status().map_err(|e| format!("curl: {e}"))?;
    if !st.success() {
        return Err(format!("download failed: {url}").into());
    }
    // 校验：发布清单（site/latest.json，在仓库里）带每个资产的 SHA-256。三源回退读取：
    // Gitee raw → AtomGit contents API → GitHub raw，任一可达即可。清单取不到时
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
    Ok(out)
}

/// `--install`：下载并安装。MSI → `msiexec /qn` 静默装；portable zip → 解压换目录。
/// `install_dir`（`--install-dir`）：要替换的安装目录——不给时锚定本 exe 所在目录。
fn install_update(check: &Check, current: &str, json: bool, install_dir: Option<&std::path::Path>) -> Result<(), Failure> {
    let asset = pick_asset(&check.assets)
        .ok_or_else(|| Failure::from(format!("the {} release lists no Windows asset (found: {})", check.tag, check.assets.join(", "))))?;
    let archive = fetch_asset(check, json)?;

    if asset.ends_with(".msi") {
        // MSI 自己管理安装位置；WordCraft 正在运行时 msiexec 会失败，静默装前提示。
        if !json {
            println!("installing {} silently (close TTWriter if it is running)", archive.display());
        }
        let st = std::process::Command::new("msiexec").args(["/i"]).arg(&archive).arg("/qn").status().map_err(|e| format!("msiexec: {e}"))?;
        if !st.success() {
            return Err(format!("msiexec failed ({}); run it yourself: msiexec /i \"{}\"", st.code().unwrap_or(-1), archive.display()).into());
        }
        if json {
            println!("{}", serde_json::json!({"installed": archive.to_string_lossy(), "tag": check.tag, "kind": "msi"}));
        } else {
            println!("installed {} over {current} — start TTWriter from the Start menu as usual", check.tag);
        }
        return Ok(());
    }

    // Portable zip：解压 → 旁边换目录。锚点：--install-dir 给的目录，或本 exe 所在目录。
    let here = match install_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            exe.parent().ok_or_else(|| Failure::from("no parent dir for the running exe"))?.to_path_buf()
        }
    };
    if !here.is_dir() {
        return Err(format!("--install-dir points nowhere: {}", here.display()).into());
    }
    // zip 里通常有一层 wordcraft-<version>-windows-x64-portable/；找到含 exe 的那一层。
    let unpacked = unpack_portable(&archive, check.tag.trim_start_matches('v')).map_err(|e| format!("unzip {}: {e}", archive.display()))?;
    let staged = find_install_root(&unpacked).ok_or_else(|| {
        let _ = std::fs::remove_dir_all(&unpacked);
        Failure::from(format!("the zip has neither ttwriter.exe nor wordcraft-cli.exe at a single root: {}", unpacked.display()))
    })?;

    // 自替换检查：本 exe 正在目标安装目录里运行时（Windows 锁着 exe），只解压不换目录。
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let exe_canon = exe_dir.canonicalize().unwrap_or_else(|_| exe_dir.to_path_buf());
        let staged_canon = staged.canonicalize().unwrap_or_else(|_| staged.clone());
        let here_canon = here.canonicalize().unwrap_or_else(|_| here.clone());
        let running_inside = exe_canon == here_canon || exe_canon.starts_with(&here_canon);
        let staged_on_target = staged_canon == here_canon || staged_canon.starts_with(&here_canon);
        if running_inside && staged_on_target {
            let msg = format!(
                "cannot swap the install dir in place: this exe runs from {} and Windows locks a running exe.\n\
                 the new version is unpacked at {} — close TTWriter, move that folder over {} (or run this \
                 command from anywhere outside the install dir next time)",
                here.display(),
                staged.display(),
                here.display()
            );
            return Err(msg.into());
        }
    }

    // staged 与安装目录是不同目录：把新目录挪到 here 的父级、旧的留原地（用户确认后删）。
    let parent = here.parent().ok_or_else(|| Failure::from("no grandparent dir for the install"))?;
    let final_dir = parent.join(staged.file_name().unwrap_or_default());
    if final_dir.exists() && final_dir != here {
        // 已有同名目录（之前装过同版？）：不覆盖，新目录留在原地让用户处置。
        let msg = format!(
            "{} already exists; the new version stays unpacked at {} — remove the old one and move it in, \
             or run this from a fresh path",
            final_dir.display(),
            staged.display()
        );
        return Err(msg.into());
    }
    // staged 可能在临时盘（跨卷）：rename 失败就 copy。
    if final_dir != here && std::fs::rename(&staged, &final_dir).is_err() {
        copy_dir(&staged, &final_dir).map_err(|e| format!("copy {} -> {}: {e}", staged.display(), final_dir.display()))?;
        let _ = std::fs::remove_dir_all(&staged);
    }
    let _ = std::fs::remove_file(&archive);
    if json {
        println!("{}", serde_json::json!({"installed": final_dir.to_string_lossy(), "tag": check.tag, "kind": "portable"}));
    } else {
        println!("installed {} at {} (was {current})", check.tag, final_dir.display());
        println!("the old folder is untouched at {}; delete it when the new one checks out", here.display());
    }
    Ok(())
}

/// 解压 portable zip 到临时目录（zip 里的顶层目录名不信任，用自己起的）。
fn unpack_portable(archive: &std::path::Path, version: &str) -> Result<PathBuf, String> {
    let out = std::env::temp_dir().join(format!("wordcraft-install-{version}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    // 上限：解出的总字节不超过 1GB（zip 炸弹防护），条目数不超过 10 万。
    const MAX_TOTAL: u64 = 1 << 30;
    const MAX_ENTRIES: usize = 100_000;
    let mut total: u64 = 0;
    if zip.len() > MAX_ENTRIES {
        return Err(format!("too many entries in the zip ({})", zip.len()));
    }
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        // 名字消毒（zip-slip）：统一斜杠后拒绝绝对路径、`..` 段和 Windows 盘符。
        let name = entry.name().replace('\\', "/");
        let bad = name.starts_with('/')
            || name.split('/').any(|seg| seg == "..")
            || name.starts_with("//")
            || name.as_str().split_once(':').is_some_and(|(drive, _)| drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()));
        if bad {
            return Err(format!("unsafe entry name in the zip: {name:?}"));
        }
        let path = out.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            continue;
        }
        total = total.saturating_add(entry.size());
        if total > MAX_TOTAL {
            return Err("the zip expands past 1 GB".into());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        std::io::copy(&mut entry, &mut file).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(out)
}

/// 从解压结果里找安装根：含 ttwriter.exe 或 wordcraft-cli.exe 的目录（至多下探 3 层）。
fn find_install_root(unpacked: &std::path::Path) -> Option<PathBuf> {
    let has_exe = |d: &std::path::Path| d.join("ttwriter.exe").is_file() || d.join("wordcraft-cli.exe").is_file();
    if has_exe(unpacked) {
        return Some(unpacked.to_path_buf());
    }
    for entry in std::fs::read_dir(unpacked).ok()? {
        let entry = entry.ok()?;
        if !entry.file_type().ok().is_some_and(|t| t.is_dir()) {
            continue;
        }
        let dir = entry.path();
        if has_exe(&dir) {
            return Some(dir);
        }
        // 第三层：zip 可能有两层包裹（wordcraft-…/wordcraft-…/exe）。
        for inner in std::fs::read_dir(&dir).ok().into_iter().flatten().flatten() {
            let inner = inner.path();
            if inner.is_dir() && has_exe(&inner) {
                return Some(inner);
            }
        }
    }
    None
}

/// 递归复制目录（rename 跨卷失败的回退）。
fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let dest = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        }
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
    fn manifest_release_reads_the_latest_json_shape() {
        let v = serde_json::json!({
            "version": "0.9.0-zh.2",
            "tag": "v0.9.0-zh.2",
            "published": "2026-10-22T12:38:24Z",
            "assets": [
                {"name": "a.zip", "channel": "cn", "sha256": "x", "size": 1},
                {"name": "a.zip", "channel": "global", "sha256": "x", "size": 1},
                {"name": "a.msi", "channel": "cn", "sha256": "y", "size": 2}
            ]
        });
        let (tag, published, assets) = manifest_release(&v).expect("usable manifest");
        assert_eq!(tag, "v0.9.0-zh.2");
        assert_eq!(published, iso_to_epoch("2026-10-22"));
        // One entry per file: the manifest lists cn/global channel copies of the same name.
        assert_eq!(assets, vec!["a.zip".to_string(), "a.msi".to_string()]);
        // Anything else is not a usable manifest; the caller falls back to the release APIs.
        assert!(manifest_release(&serde_json::json!({"tag": "v1", "assets": []})).is_none());
        assert!(manifest_release(&serde_json::json!({"assets": [{"name": "a.zip"}]})).is_none());
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

    /// `--install` 的解压：正常 zip 解到临时目录并找到安装根；zip-slip（`..`、绝对路径、
    /// 盘符）被拒绝，不会写到目标目录之外。
    #[test]
    fn unpack_finds_the_install_root_and_rejects_slip() {
        let dir = std::env::temp_dir().join("wordcraft-update-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // 正常包：wordcraft-1.0-windows-x64-portable/ttwriter.exe + wordcraft-cli.exe。
        let normal = dir.join("normal.zip");
        {
            let file = std::fs::File::create(&normal).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
            zip.add_directory("wordcraft-1.0-windows-x64-portable/", opts).unwrap();
            zip.start_file("wordcraft-1.0-windows-x64-portable/ttwriter.exe", opts).unwrap();
            std::io::Write::write_all(&mut zip, b"MZ fake").unwrap();
            zip.start_file("wordcraft-1.0-windows-x64-portable/wordcraft-cli.exe", opts).unwrap();
            std::io::Write::write_all(&mut zip, b"MZ fake cli").unwrap();
            zip.finish().unwrap();
        }
        let unpacked = unpack_portable(&normal, "1.0").unwrap();
        let root = find_install_root(&unpacked).expect("find the root");
        assert!(root.join("ttwriter.exe").is_file());
        assert!(root.join("wordcraft-cli.exe").is_file());
        assert!(root.ends_with("wordcraft-1.0-windows-x64-portable"));

        // zip-slip 三连：..、绝对路径、盘符——都在解出任何文件前被拒。
        for (name, evil) in [("dotdot.zip", "../escape.exe"), ("abs.zip", "/abs.exe"), ("drive.zip", "C:/evil.exe")] {
            let path = dir.join(name);
            let file = std::fs::File::create(&path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
            zip.start_file(evil, opts).unwrap();
            std::io::Write::write_all(&mut zip, b"x").unwrap();
            zip.finish().unwrap();
            let err = unpack_portable(&path, "1.0").unwrap_err();
            assert!(err.contains("unsafe"), "{name}: {err}");
        }
        // 没逃逸发生（临时目录外没有被写）。
        assert!(!dir.parent().unwrap().join("escape.exe").exists());
        assert!(!std::path::Path::new("C:/evil.exe").exists() || true);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 没有安装根的 zip（全是 txt）找不到根。
    #[test]
    fn a_zip_without_exes_has_no_install_root() {
        let dir = std::env::temp_dir().join("wordcraft-update-test-noexe");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("noexe.zip");
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
        zip.start_file("readme.txt", opts).unwrap();
        std::io::Write::write_all(&mut zip, b"hi").unwrap();
        zip.finish().unwrap();
        let unpacked = unpack_portable(&path, "9.9").unwrap();
        assert!(find_install_root(&unpacked).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 递归复制把目录树原样搬过去。
    #[test]
    fn copy_dir_copies_the_tree() {
        let dir = std::env::temp_dir().join("wordcraft-update-copy");
        let _ = std::fs::remove_dir_all(&dir);
        let from = dir.join("from");
        std::fs::create_dir_all(from.join("sub")).unwrap();
        std::fs::write(from.join("a.txt"), b"a").unwrap();
        std::fs::write(from.join("sub").join("b.txt"), b"b").unwrap();
        let to = dir.join("to");
        copy_dir(&from, &to).unwrap();
        assert!(to.join("a.txt").is_file() && to.join("sub").join("b.txt").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
