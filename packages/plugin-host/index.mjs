#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import path from "node:path";

async function readRequest() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const line = Buffer.concat(chunks).toString("utf8").trim();
  if (!line) throw new Error("plugin host expected one JSON request on standard input");
  return JSON.parse(line);
}

async function main() {
  const request = await readRequest();
  const manifestPath = path.join(request.pluginDir, "thinking-computer-plugin.json");
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  if (request.action === "describe") return process.stdout.write(`${JSON.stringify({ ok: true, manifest })}\n`);
  if (request.action !== "invoke") throw new Error(`unsupported plugin action: ${request.action}`);
  const module = await import(pathToFileURL(path.resolve(request.pluginDir, manifest.entry)).href);
  const tools = module.tools ?? module.default;
  const handler = tools?.[request.tool];
  if (typeof handler !== "function") throw new Error(`plugin does not export tool: ${request.tool}`);
  process.stdout.write(`${JSON.stringify({ ok: true, result: await handler({ args: request.args ?? {}, context: request.context ?? {} }) })}\n`);
}

main().catch((error) => { process.stdout.write(`${JSON.stringify({ ok: false, error: error.message })}\n`); process.exitCode = 1; });

