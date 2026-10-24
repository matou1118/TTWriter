# 发布清单（Windows，GNU 工具链）

> 本机 MSVC 链接器不可用，官方发布走 GNU 工具链。一条命令出全部资产。

## 日常发布

```powershell
# 0. 门禁全绿再出包：cargo xtask ci（fmt / clippy / test / assets / layers / wasm / skills）
# 1. 版本号：改 [workspace.package] version 并提交（脚本会拒绝与 WORDCRAFT_VERSION 漂移）
# 2. 构建 + 打包 + 清单（约 10 分钟）
powershell -File packaging/windows/package-gnu.ps1

# 产物在 dist/release-gnu/：
#   wordcraft-<版本>-windows-x64-portable.zip   便携包（ttwriter.exe + wordcraft-cli.exe + 许可）
#   latest.json                                 更新清单（双通道 cn/global + SHA-256）
```

脚本自动做：版本一致性检查 → cargo build（`WORDCRAFT_REQUIRE_WINRES=1`，图标缺失即失败）→
PE 头检查（x64 / GUI+console 子系统）→ 暂存（`ttwriter.exe` 改名 + 许可 + 使用说明）→
zip → CLI `--version` 冒烟 → manifest。

## 发布步骤

1. `package-gnu.ps1` 出 zip + latest.json（本地）
2. `git checkout -- site/latest.json`——**先别提交**，等 release 传完再提交（见下）
3. GitHub（matou1118/ttwriter）与 AtomGit（matou118/ttwriter）各传一个 release：
   - tag：`v<版本>`（zip 名里的版本要一致）
   - 资产：zip（两处同一文件，SHA-256 相同）
4. **传完资产后**把 `site/latest.json` 的更新提交进仓库并推送（Gitee/GitHub 同步）——
   更新器从 Gitee raw 读它；没传资产就提交清单，用户会下载到 404
5. 验证：干净机器上 `wordcraft-cli update --check` → `--install`，跑一次 `--version`

## 网页版（GitHub Pages /app/）

release 发布会自动触发 `pages.yml` 重编网页版（wasm + 内置中文字体，约 10 分钟）。
手动触发：Actions → Pages → Run workflow。

- **本地预发验证**：`powershell apps/wordcraft-web/build-web.ps1`（trunk + wasm-bindgen
  预编译二进制在 `target-gnu/trunk-bin/`；GNU 工具链自动设；产物在 `dist/web`）
- **首次加载约 58MB**（wasm 含 Noto 中文字形）——国内网络慢是常态，页面有下载进度
  与 60 秒超时指引；装成 PWA 后走缓存离线可用
- **craft-fonts 是可选构建输入**：CI checkout `storytold/craft-fonts` 并设
  `CRAFT_FONTS_DIR`；本地用仓库旁的 `craft-fonts/` 目录（不进仓库）。**不嵌字体的
  后果是中文界面全是问号**——`crates/fonts/build.rs` 的 wasm 过滤规则只保留 UI 字形
  （BIZ UDPGothic + Hans Regular/Bold），文档字体（仿宋/文楷）不进 wasm（体积权衡）
- **PWA 资产**（manifest/sw.js/图标）由 pages.yml 的 assemble 步骤与 build-web.ps1
  纯拷贝——**两处清单必须同步**（trunk 的 copy-file 会写 1 字节假文件，别用它）
- 推送含 `apps/wordcraft-web/` 改动的提交后**记得手动触发**（pages.yml 的 push 触发
  只看 `site/**` 等路径，web 源码变更不在其内——避免每次 push 都跑 10 分钟 wasm 构建）

## 回滚

release 出问题：把 `site/latest.json` 回退到上一个好版本的清单（tag、资产名、SHA-256 都
改回去）并提交——更新器看清单不看 release 列表，清单指哪用户去哪。

## 语义决定（别丢）

- **便携包旁路换目录**：`update --install` 把新版装到旧版旁边，旧目录永远不动，用户手动删。
  绝不自动删除用户目录（哪怕是"旧版"）。
- **MSI 静默装**（如果将来 MSVC 可用出 MSI）：`msiexec /i /qn`，失败时报完整命令让用户自跑。
- **语言包/技能编译进二进制**：zip 里没有数据文件，升级 = 换 exe，无迁移。
- **`WORDCRAFT_UPDATE_MANIFEST`** 环境变量可覆盖清单源（自建镜像/测试），资产 `url` 字段
  优先于按源拼接。
