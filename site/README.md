# 官网

中英文两页：`index.html`（简体中文，`/`）与 `en.html`（English，`/en.html`），页头互相切换。
发布地址 `https://matou1118.github.io/TTWriter/`。
设计令牌取自 [www.ttread.cn](https://www.ttread.cn)（蜻蜓伴读）：朱砂强调色、暖白底、系统字体栈、1120px 版心。没有引入任何第三方 CSS / JS，字体全走系统栈，零网络请求。

## 本地预览

页面里的图片路径是 `images/zh/…` 与 `icon/…`，发布时由 workflow 拷进来（截图只有 `docs/images/` 一份，不在仓库里存两遍）。本地预览就跑同样的组装：

```bash
mkdir -p _site && cp -r site/. _site/
mkdir -p _site/images _site/icon
cp -r docs/images/. _site/images/
cp assets/app-icon/hicolor/256x256/apps/ai.storyteller.wordcraft.png _site/icon/icon.png
cp assets/app-icon/wordcraft.svg _site/icon/icon.svg
python3 -m http.server -d _site 8000
```

## 发布

`.github/workflows/pages.yml` 在 `site/**`、`docs/images/**`、`assets/app-icon/**` 变动时跑。
仓库设置里把 **Settings → Pages → Source** 选成 **GitHub Actions** 一次即可。

## 想挂到 www.ttread.cn 下面

`site/` 就是纯静态文件（HTML + CSS + JS，没有构建步骤），直接整份拷到 ttread.cn 的任意子目录即可；
页面内都是相对路径，换个目录不用改。注意 ttread.cn 有 CSP，`style-src` 允许 `unsafe-inline` 但 `script-src` 只放行同源加一个哈希——
本站的脚本在 `app.js` 里、page 内没有内联脚本，所以挂过去只需把 `app.js` 加进 `script-src 'self'` 的允许范围（同源脚本本来就在 `'self'` 里，通常不用改）。

## 改内容

`index.html` 是单页，五个区块：这是什么 / 中文排版层 / 相对上游迭代了什么 / 装上试试 / 与上游的关系。
截图在 `docs/images/zh/`，重新生成的方法看仓库根目录的 `test-all.ps1` 与 `README.md`。
