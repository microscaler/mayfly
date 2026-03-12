/**
 * Render Mermaid diagrams in mdbook: load Mermaid from CDN and replace
 * each code.language-mermaid block with the rendered diagram.
 */
(function () {
  function renderMermaid() {
    var codes = document.querySelectorAll("code.language-mermaid");
    if (codes.length === 0) return;

    var fragments = [];
    codes.forEach(function (code) {
      var pre = code.parentElement;
      if (!pre || pre.tagName !== "PRE") return;
      var div = document.createElement("div");
      div.className = "mermaid";
      div.textContent = code.textContent.trim();
      pre.parentNode.insertBefore(div, pre.nextSibling);
      pre.style.display = "none";
      fragments.push(div);
    });

    if (window.mermaid) {
      window.mermaid.run({ nodes: fragments });
    }
  }

  function loadMermaid() {
    var script = document.createElement("script");
    script.src =
      "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js";
    script.async = false;
    script.onload = function () {
      window.mermaid.initialize({
        startOnLoad: false,
        theme: "base",
        securityLevel: "loose",
        themeVariables: {
          darkMode: false,
          background: "#fafafa",
          /* Box fills: distinct from background so boxes read clearly */
          primaryColor: "#e2e8f0",
          primaryTextColor: "#1e293b",
          primaryBorderColor: "#64748b",
          secondaryColor: "#e2e8f0",
          secondaryTextColor: "#1e293b",
          secondaryBorderColor: "#64748b",
          tertiaryColor: "#f1f5f9",
          tertiaryTextColor: "#1e293b",
          tertiaryBorderColor: "#64748b",
          mainBkg: "#e2e8f0",
          textColor: "#1e293b",
          nodeTextColor: "#1e293b",
          titleColor: "#1e293b",
          edgeLabelBackground: "#e2e8f0",
          lineColor: "#334155",
          /* Sequence: participant/actor boxes and activation bars */
          actorBkg: "#e2e8f0",
          actorBorder: "#64748b",
          actorLineColor: "#334155",
          actorTextColor: "#1e293b",
          signalColor: "#1e293b",
          signalTextColor: "#1e293b",
          labelBoxBkgColor: "#e2e8f0",
          labelBoxBorderColor: "#64748b",
          labelTextColor: "#1e293b",
          loopTextColor: "#1e293b",
          noteBkgColor: "#fef3c7",
          noteTextColor: "#1e293b",
          noteBorderColor: "#64748b",
          activationBorderColor: "#64748b",
          activationBkgColor: "#cbd5e1",
          sequenceNumberColor: "#334155",
        },
      });
      renderMermaid();
    };
    script.onerror = function () {
      console.warn("Mermaid failed to load; diagrams will show as code.");
    };
    document.head.appendChild(script);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", loadMermaid);
  } else {
    loadMermaid();
  }
})();
