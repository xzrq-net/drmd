import { renderDiagrams } from "./diagrams.js";

(async () => {
  const key = "drmd-scroll:" + location.pathname;
  const saved = sessionStorage.getItem(key);
  const es = new EventSource("/__reload");
  es.onmessage = () => location.reload();

  // Diagram layout changes page height; restore scroll after it settles.
  await renderDiagrams();
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
})();
