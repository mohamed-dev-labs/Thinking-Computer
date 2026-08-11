import worker from "./worker.js";

const response = await worker.fetch(new Request("https://example.invalid/"));
const page = await response.text();

if (response.status !== 200) throw new Error(`expected HTTP 200, received ${response.status}`);
if (!page.includes("THINKING COMPUTER") || !page.includes("Get started")) {
  throw new Error("marketing page is missing its required identity or primary action");
}

console.log(JSON.stringify({ status: response.status, bytes: page.length }));
