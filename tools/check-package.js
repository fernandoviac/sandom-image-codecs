// Checks the npm package file list: every file a consumer needs is present,
// and no runtime source ships.
const expected = new Set([
  "package.json",
  "README.md",
  "DESIGN.md",
  "LICENSE",
  "NOTICE",
  "dist/jpeg.wasm",
  "dist/webp.wasm",
  "docs/legal/image-decoders.md",
]);

const result = await new Deno.Command("npm", {
  args: ["pack", "--dry-run", "--json", "--ignore-scripts"],
  stdout: "piped",
  stderr: "piped",
}).output();
if (!result.success) {
  throw new Error(new TextDecoder().decode(result.stderr).trim());
}
const report = JSON.parse(new TextDecoder().decode(result.stdout));
const files = new Set(report[0]?.files?.map(({ path }) => path));
for (const path of expected) {
  if (!files.has(path)) throw new Error(`Package is missing ${path}`);
}
for (const path of files) {
  if (/\.(?:js|mjs|cjs|ts|tsx|jsx|c|cc|cpp|h|rs)$/.test(path)) {
    throw new Error(`Package contains prohibited runtime source ${path}`);
  }
}
for (const name of ["jpeg", "webp"]) {
  await WebAssembly.compile(await Deno.readFile(`dist/${name}.wasm`));
}
console.log(
  JSON.stringify({ kind: "sandom/image-codecs-package-check", files: files.size }),
);
