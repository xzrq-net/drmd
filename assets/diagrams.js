export async function renderDiagrams() {
  const blocks = document.querySelectorAll("pre > code.language-mermaid");
  if (blocks.length === 0) return;

  function showError(block, error) {
    const message = document.createElement("pre");
    message.className = "mermaid-error";
    message.textContent = `Mermaid: ${error.message ?? error}`;
    block.parentElement.before(message);
  }

  try {
    await new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = "/__assets/mermaid.min.js";
      script.onload = resolve;
      script.onerror = () => reject(new Error("could not load the diagram renderer"));
      document.head.append(script);
    });
  } catch (error) {
    for (const block of blocks) showError(block, error);
    return;
  }

  const mermaid = window.mermaid;
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    suppressErrorRendering: true,
  });

  for (const [index, block] of blocks.entries()) {
    const source = block.parentElement;
    const diagram = document.createElement("div");
    try {
      const { svg, bindFunctions } = await mermaid.render(
        `drmd-mermaid-${index}`,
        block.textContent,
      );
      diagram.className = "mermaid-diagram";
      diagram.innerHTML = svg;
      source.replaceWith(diagram);
      bindFunctions?.(diagram);
    } catch (error) {
      diagram.replaceWith(source);
      showError(block, error);
    }
  }
}
