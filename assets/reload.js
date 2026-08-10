(() => {
  const key = "drmd-scroll:" + location.pathname;
  const saved = sessionStorage.getItem(key);
  if (saved !== null) scrollTo(0, parseFloat(saved));
  let ticking = false;
  addEventListener(
    "scroll",
    () => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(() => {
        sessionStorage.setItem(key, String(scrollY));
        ticking = false;
      });
    },
    { passive: true },
  );
  const es = new EventSource("/__reload");
  es.onmessage = () => location.reload();
})();
