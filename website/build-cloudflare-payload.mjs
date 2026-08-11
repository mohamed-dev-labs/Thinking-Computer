import { readFile, writeFile } from "node:fs/promises";

const source = await readFile(new URL("./worker.js", import.meta.url), "utf8");
const metadata = {
  main_module: "worker.js",
  compatibility_date: "2025-02-01",
  compatibility_flags: ["nodejs_compat"],
};

const code = `async () => {
  const boundary = "----ThinkingComputer" + Date.now();
  const metadata = ${JSON.stringify(metadata)};
  const source = ${JSON.stringify(source)};
  const body = [
    "--" + boundary,
    "Content-Disposition: form-data; name=\\\"metadata\\\"",
    "Content-Type: application/json",
    "",
    JSON.stringify(metadata),
    "--" + boundary,
    "Content-Disposition: form-data; name=\\\"worker.js\\\"; filename=\\\"worker.js\\\"",
    "Content-Type: application/javascript+module",
    "",
    source,
    "--" + boundary + "--",
    ""
  ].join("\\r\\n");
  return cloudflare.request({
    method: "PUT",
    path: \`/accounts/\${accountId}/workers/scripts/tc\`,
    body,
    contentType: \`multipart/form-data; boundary=\${boundary}\`,
    rawBody: true
  });
}`;

await writeFile("/tmp/thinking-computer-cloudflare-deploy.json", JSON.stringify({ code }));
