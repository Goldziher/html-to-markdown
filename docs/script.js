import init, { convert } from "./html_to_markdown_wasm.js";

// State
let wasmInitialized = false;

// DOM Elements
const htmlInput = document.getElementById("htmlInput");
const markdownOutput = document.getElementById("markdownOutput");
const convertBtn = document.getElementById("convertBtn");
const copyBtn = document.getElementById("copyBtn");
const clearBtn = document.getElementById("clearBtn");
const statusEl = document.getElementById("status");
const wasmStatusEl = document.getElementById("wasmStatus");

// Initialize WASM module
async function initWasm() {
  try {
    await init();
    wasmInitialized = true;
    wasmStatusEl.textContent = "✓ WASM module loaded";
    wasmStatusEl.classList.add("loaded");
    statusEl.textContent = "Ready to convert!";
    statusEl.classList.add("success");

    // Auto-convert the example on load
    performConversion();
  } catch (error) {
    console.error("Failed to initialize WASM:", error);
    wasmStatusEl.textContent = "✗ Failed to load WASM";
    wasmStatusEl.classList.add("error");
    statusEl.textContent = "Error: Failed to load WASM module";
    statusEl.classList.add("error");
    convertBtn.disabled = true;
  }
}

// Perform HTML to Markdown conversion
function performConversion() {
  if (!wasmInitialized) {
    statusEl.textContent = "WASM module not initialized yet...";
    statusEl.className = "status";
    return;
  }

  const html = htmlInput.value.trim();

  if (!html) {
    markdownOutput.textContent = "";
    statusEl.textContent = "Please enter some HTML to convert";
    statusEl.className = "status";
    return;
  }

  try {
    const startTime = performance.now();

    // Convert HTML to Markdown
    const markdown = convert(html, null);

    const endTime = performance.now();
    const duration = (endTime - startTime).toFixed(2);

    // Display result
    markdownOutput.textContent = markdown;
    statusEl.textContent = `✓ Converted in ${duration}ms`;
    statusEl.className = "status success";

    // Reset copy button if it was in copied state
    copyBtn.classList.remove("copied");
  } catch (error) {
    console.error("Conversion error:", error);
    markdownOutput.textContent = "";
    statusEl.textContent = `Error: ${error.message}`;
    statusEl.className = "status error";
  }
}

// Copy markdown to clipboard
async function copyToClipboard() {
  const markdown = markdownOutput.textContent;

  if (!markdown) {
    statusEl.textContent = "Nothing to copy";
    statusEl.className = "status";
    return;
  }

  try {
    await navigator.clipboard.writeText(markdown);
    copyBtn.classList.add("copied");
    statusEl.textContent = "✓ Copied to clipboard!";
    statusEl.className = "status success";

    // Reset button after 2 seconds
    setTimeout(() => {
      copyBtn.classList.remove("copied");
    }, 2000);
  } catch (error) {
    console.error("Failed to copy:", error);
    statusEl.textContent = "Failed to copy to clipboard";
    statusEl.className = "status error";
  }
}

// Clear input
function clearInput() {
  htmlInput.value = "";
  markdownOutput.textContent = "";
  statusEl.textContent = "Input cleared";
  statusEl.className = "status";
  htmlInput.focus();
}

// Event Listeners
convertBtn.addEventListener("click", performConversion);
copyBtn.addEventListener("click", copyToClipboard);
clearBtn.addEventListener("click", clearInput);

// Convert on Enter (Ctrl/Cmd + Enter)
htmlInput.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
    e.preventDefault();
    performConversion();
  }
});

// Auto-convert on input (debounced)
let debounceTimer;
htmlInput.addEventListener("input", () => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (wasmInitialized && htmlInput.value.trim()) {
      performConversion();
    }
  }, 500);
});

// Initialize on page load
initWasm();
