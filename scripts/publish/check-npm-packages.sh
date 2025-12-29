#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"

check_pkg() {
	pkg="$1"
	node - "$pkg" "$version" <<'NODE'
const https = require("https");
const pkg = process.argv[1];
const version = process.argv[2];

function fetch(url) {
  return new Promise((resolve) => {
    https
      .get(url, (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          try {
            const body = JSON.parse(data);
            const exists = body.versions && body.versions[version] ? "true" : "false";
            resolve(exists);
          } catch (err) {
            resolve("false");
          }
        });
      })
      .on("error", () => resolve("false"));
  });
}

fetch(`https://registry.npmjs.org/${encodeURIComponent(pkg)}`)
  .then((exists) => console.log(exists))
  .catch(() => console.log("false"));
NODE
}

node_exists=$(check_pkg "@kreuzberg/html-to-markdown-node")
wasm_exists=$(check_pkg "@kreuzberg/html-to-markdown-wasm")

{
	echo "node_exists=${node_exists}"
	echo "wasm_exists=${wasm_exists}"
} >>"${GITHUB_OUTPUT}"
