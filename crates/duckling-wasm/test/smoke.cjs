// Smoke test: the WASM build must return exactly what the native server returns.
// Expected strings were captured from `duckling-server` (itself byte-compared to the Haskell server).
// Usage: node test/smoke.cjs [pkg-dir]   (pkg-dir defaults to ../pkg, from `wasm-bindgen --target nodejs`)
const fs = require("fs");
const path = require("path");
const assert = require("assert");

const pkg = path.resolve(process.argv[2] || path.join(__dirname, "..", "pkg"));
const { Duckling } = require(path.join(pkg, "duckling_wasm.js"));

const tzdata = path.resolve(__dirname, "..", "..", "duckling", "tests", "tz", "tzdata");
const d = new Duckling();
d.addTimeZone("America/Los_Angeles", fs.readFileSync(path.join(tzdata, "America", "Los_Angeles")));

const REF = 1700000000000;
const cases = [
  [
    { text: "tomorrow at 5pm", lang: "en", tz: "America/Los_Angeles", reftime: REF, dims: ["time", "number", "email"] },
    '[{"body":"tomorrow at 5pm","dim":"time","end":15,"latent":false,"start":0,"value":{"grain":"hour","type":"value","value":"2023-11-15T17:00:00.000-08:00","values":[{"grain":"hour","type":"value","value":"2023-11-15T17:00:00.000-08:00"}]}}]',
  ],
  [
    { text: "I have 3 apples and email a@b.com", lang: "en", tz: "America/Los_Angeles", reftime: REF, dims: ["time", "number", "email"] },
    '[{"body":"3","dim":"number","end":8,"latent":false,"start":7,"value":{"type":"value","value":3}},{"body":"a@b.com","dim":"email","end":33,"latent":false,"start":26,"value":{"type":"value","value":"a@b.com"}}]',
  ],
  [
    { text: "from 9am to 5pm next friday", lang: "en", tz: "America/Los_Angeles", reftime: REF, dims: ["time", "number", "email"] },
    '[{"body":"from 9am to 5pm next friday","dim":"time","end":27,"latent":false,"start":0,"value":{"from":{"grain":"hour","value":"2023-11-24T09:00:00.000-08:00"},"to":{"grain":"hour","value":"2023-11-24T18:00:00.000-08:00"},"type":"interval","values":[{"from":{"grain":"hour","value":"2023-11-24T09:00:00.000-08:00"},"to":{"grain":"hour","value":"2023-11-24T18:00:00.000-08:00"},"type":"interval"}]}}]',
  ],
];

for (const [req, expected] of cases) {
  assert.strictEqual(d.parse(JSON.stringify(req)), expected, req.text);
}

const targets = JSON.parse(d.targets());
assert.ok(Array.isArray(targets.en) && targets.en.includes("time"));

assert.throws(() => d.parse(JSON.stringify({ lang: "en" })), /text/);
console.log(`wasm smoke test OK (${cases.length} parses)`);
