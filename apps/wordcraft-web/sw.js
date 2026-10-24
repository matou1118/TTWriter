// 蜻蜓文档网页版的 Service Worker：离线可用 + 版本更新提示。
//
// 策略（wasm 42MB，预缓存清单写死哈希名不现实——trunk 每次构建变哈希）：
//   1. 运行时缓存（stale-while-revalidate）：fetch 到的同源 GET（index/JS/wasm/图标）
//      一律进缓存；先回缓存（快、离线可用），后台再更新。
//   2. 版本哨兵：index.html 永远走网络优先（离线才用缓存）——新版本上线时页面
//      自然而然拿到新 index，新 index 引用新哈希的 JS/wasm，旧缓存自然废弃。
//   3. 旧缓存回收：激活时清掉非当前缓存名的所有 Cache。
//   4. 更新提示：新 SW 安装后 skipWaiting + controllerchange 时刷新页面
//      （sw-register.js 里做，用户正在打字时不打断——见那边的注释）。
const CACHE = "ttwriter-v1";

self.addEventListener("install", (e) => {
  // 不预缓存大文件；安装即激活（激活时清旧缓存）。
  self.skipWaiting();
});

self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (e) => {
  const req = e.request;
  if (req.method !== "GET") return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return; // 只管同源

  // 导航请求（打开页面）：网络优先，离线回缓存——保证新版本尽快上线。
  if (req.mode === "navigate") {
    e.respondWith(
      fetch(req)
        .then((r) => {
          const copy = r.clone();
          caches.open(CACHE).then((c) => c.put(req, copy));
          return r;
        })
        .catch(() => caches.match(req, { ignoreSearch: true }).then((m) => m || Response.error()))
    );
    return;
  }

  // 资产（JS/wasm/图标）：stale-while-revalidate——先缓存后更新，离线照样跑。
  e.respondWith(
    caches.match(req).then((cached) => {
      const refresh = fetch(req)
        .then((r) => {
          if (r.ok) {
            const copy = r.clone();
            caches.open(CACHE).then((c) => c.put(req, copy));
          }
          return r;
        })
        .catch(() => cached);
      return cached || refresh;
    })
  );
});
