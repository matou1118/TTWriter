// 主题切换 + 入场上浮。没有 eval / innerHTML / document.write，和 ttread.cn 一样只做 DOM 操作。
(function () {
  "use strict";

  var KEY = "ttwriter-theme";
  var root = document.documentElement;
  var meta = document.querySelector('meta[name="theme-color"]');
  var LIGHT = "#f7f5f0";
  var DARK = "#0d1017";

  function apply(theme) {
    root.setAttribute("data-theme", theme);
    if (meta) meta.setAttribute("content", theme === "dark" ? DARK : LIGHT);
  }

  // 先按已存的选择，其次跟系统。
  var saved = null;
  try { saved = localStorage.getItem(KEY); } catch (e) { /* 隐私模式下读不到，按系统走 */ }
  var prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
  apply(saved === "dark" || saved === "light" ? saved : (prefersDark ? "dark" : "light"));

  var btn = document.getElementById("theme");
  if (btn) {
    btn.addEventListener("click", function () {
      var next = root.getAttribute("data-theme") === "dark" ? "light" : "dark";
      apply(next);
      try { localStorage.setItem(KEY, next); } catch (e) { /* 存不下也不影响这次切换 */ }
    });
  }

  var reveal = document.querySelectorAll(".reveal");
  var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduce || !("IntersectionObserver" in window)) {
    return; // 保持可见
  }
  reveal.forEach(function (el) { el.classList.add("off"); });
  var io = new IntersectionObserver(function (entries) {
    entries.forEach(function (e) {
      if (e.isIntersecting) {
        e.target.classList.remove("off");
        io.unobserve(e.target);
      }
    });
  }, { rootMargin: "0px 0px -8% 0px" });
  reveal.forEach(function (el) { io.observe(el); });
})();
