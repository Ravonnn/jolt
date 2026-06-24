/* Interactive Jolt code cells for mdBook — requires jolt-learn-runner on :3847 */

(function () {
  const RUNNER = "http://127.0.0.1:3847";
  const FALLBACK_CMD =
    "cargo build -p jolt-cli && ./target/debug/jolt run --interpret program.jolt";

  function enhanceBlocks() {
    document.querySelectorAll("pre code.language-jolt").forEach((code) => {
      const pre = code.parentElement;
      if (!pre || pre.dataset.joltEnhanced) return;
      const text = code.textContent || "";
      const runnable =
        code.classList.contains("runnable") ||
        pre.classList.contains("runnable") ||
        /runnable/.test(code.className);
      if (!runnable) return;
      pre.dataset.joltEnhanced = "1";

      const wrap = document.createElement("div");
      wrap.className = "jolt-cell";
      pre.parentNode.insertBefore(wrap, pre);
      wrap.appendChild(pre);

      const ta = document.createElement("textarea");
      ta.className = "jolt-editor";
      ta.value = text.trimEnd();
      ta.spellcheck = false;
      wrap.insertBefore(ta, pre);
      pre.style.display = "none";

      const toolbar = document.createElement("div");
      toolbar.className = "jolt-toolbar";
      wrap.appendChild(toolbar);

      const out = document.createElement("pre");
      out.className = "jolt-output";
      wrap.appendChild(out);

      const original = ta.value;

      function setOutput(kind, msg) {
        out.className = "jolt-output " + kind;
        out.textContent = msg;
      }

      async function api(path, body) {
        const res = await fetch(RUNNER + path, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        });
        return res.json();
      }

      async function runCommand(command) {
        setOutput("pending", "Running…");
        try {
          const data = await api("/api/v1/run", {
            source: ta.value,
            command,
          });
          if (data.error) {
            setOutput("error", data.error + "\n\nLocal fallback:\n" + FALLBACK_CMD);
            return;
          }
          const parts = [];
          if (data.stdout) parts.push(data.stdout);
          if (data.stderr) parts.push(data.stderr);
          const text = parts.join("\n").trim() || "(no output)";
          setOutput(data.success ? "ok" : "error", text);
        } catch (_e) {
          setOutput(
            "offline",
            "Learn runner offline. Start it with:\n\n  cargo xtask learn serve\n\nOr run manually:\n  " +
              FALLBACK_CMD
          );
        }
      }

      function btn(label, command) {
        const b = document.createElement("button");
        b.type = "button";
        b.textContent = label;
        b.addEventListener("click", () => runCommand(command));
        toolbar.appendChild(b);
      }

      btn("Run", "run");
      btn("Check", "check");
      btn("Reset", null);
      toolbar.lastChild.addEventListener("click", () => {
        ta.value = original;
        setOutput("", "");
      });

      const copy = document.createElement("button");
      copy.type = "button";
      copy.textContent = "Copy CLI";
      copy.addEventListener("click", () => {
        navigator.clipboard.writeText(
          "cargo build -p jolt-cli && ./target/debug/jolt run --interpret - <<'EOF'\n" +
            ta.value +
            "\nEOF"
        );
      });
      toolbar.appendChild(copy);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", enhanceBlocks);
  } else {
    enhanceBlocks();
  }
})();
