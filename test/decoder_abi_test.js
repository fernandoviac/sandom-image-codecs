// Raw-ABI tests for dist/jpeg.wasm and dist/webp.wasm. Each test drives the
// five-export call order a host binding uses (alloc, write bytes, decode,
// read width/height/pixels, release) without any host binding in between.

const FIXTURES = new URL("./fixtures/", import.meta.url);
const DIST = new URL("../dist/", import.meta.url);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function instantiate(name) {
  const module = await WebAssembly.compile(await Deno.readFile(new URL(name, DIST)));
  return { module, exports: (await WebAssembly.instantiate(module, {})).exports };
}

function fixture(name) {
  return Deno.readFile(new URL(name, FIXTURES));
}

// Returns { status } for a failed decode, or { status: 0, width, height,
// pixels } with the pixels copied out before release.
function decode(exports, bytes) {
  const { alloc, decode, width, height, pixels, release, memory } = exports;
  const inputAddress = alloc(bytes.length);
  new Uint8Array(memory.buffer, inputAddress, bytes.length).set(bytes);
  const status = decode();
  if (status !== 0) {
    const frameCount = exports.frames ? exports.frames() : 0;
    release();
    return { status, frameCount };
  }
  const decodedWidth = width();
  const decodedHeight = height();
  const copied = new Uint8Array(
    new Uint8Array(memory.buffer, pixels(), decodedWidth * decodedHeight * 4),
  );
  release();
  return { status, width: decodedWidth, height: decodedHeight, pixels: copied };
}

function alphaValues(pixels) {
  const alphas = new Set();
  for (let index = 3; index < pixels.length; index += 4) alphas.add(pixels[index]);
  return alphas;
}

for (const name of ["jpeg.wasm", "webp.wasm"]) {
  Deno.test(`${name}: the module imports nothing`, async () => {
    const { module } = await instantiate(name);
    const imports = WebAssembly.Module.imports(module);
    assert(imports.length === 0, `${name} imports ${JSON.stringify(imports)}`);
  });

  Deno.test(`${name}: decode before alloc returns -1`, async () => {
    const { exports } = await instantiate(name);
    const status = exports.decode();
    assert(status === -1, `expected -1, got ${status}`);
  });

  Deno.test(`${name}: status messages name the failure and fall back for an unknown status`, async () => {
    const { exports } = await instantiate(name);
    const text = (status) => new TextDecoder().decode(
      new Uint8Array(exports.memory.buffer, exports.status_message(status), exports.status_message_length(status)));
    assert(text(-1) === "internal error: decode() called before alloc()", `-1: ${text(-1)}`);
    assert(text(-2).startsWith("not a decodable "), `-2: ${text(-2)}`);
    assert(text(-3).startsWith("internal error: decoder returned a buffer"), `-3: ${text(-3)}`);
    assert(text(7) === "unknown decoder status", `7: ${text(7)}`);
  });
}

Deno.test("jpeg.wasm: a baseline JPEG decodes to opaque RGBA8 at its intrinsic size", async () => {
  const { exports } = await instantiate("jpeg.wasm");
  const result = decode(exports, await fixture("test-photo-baseline.jpg"));
  assert(result.status === 0, `decode returned ${result.status}`);
  assert(result.width === 120 && result.height === 180, `expected 120x180, got ${result.width}x${result.height}`);
  const alphas = alphaValues(result.pixels);
  assert(alphas.size === 1 && alphas.has(255), `expected alpha 255 only, got [${[...alphas]}]`);
});

Deno.test("jpeg.wasm: progressive and baseline encodings of one photo decode alike", async () => {
  const { exports } = await instantiate("jpeg.wasm");
  const baseline = decode(exports, await fixture("test-photo-baseline.jpg"));
  const progressive = decode(exports, await fixture("test-photo-progressive.jpg"));
  assert(progressive.status === 0, `progressive decode returned ${progressive.status}`);
  assert(progressive.width === baseline.width && progressive.height === baseline.height,
    `progressive ${progressive.width}x${progressive.height}, baseline ${baseline.width}x${baseline.height}`);
  // A decoder that stops after the first progressive scan yields a blurred
  // image far outside this bound.
  let maximumDelta = 0;
  for (let index = 0; index < baseline.pixels.length; index++) {
    maximumDelta = Math.max(maximumDelta, Math.abs(baseline.pixels[index] - progressive.pixels[index]));
  }
  assert(maximumDelta <= 12, `baseline vs progressive maximum delta ${maximumDelta}`);
});

Deno.test("jpeg.wasm: a grayscale JPEG expands to RGBA with equal channels", async () => {
  const { exports } = await instantiate("jpeg.wasm");
  const result = decode(exports, await fixture("test-photo-grayscale.jpg"));
  assert(result.status === 0, `decode returned ${result.status}`);
  assert(result.pixels.length === result.width * result.height * 4, "grayscale output is not RGBA8");
  for (let index = 0; index < result.pixels.length; index += 4) {
    const [red, green, blue] = result.pixels.subarray(index, index + 3);
    assert(red === green && green === blue, `pixel at byte ${index} is ${red},${green},${blue}`);
  }
});

Deno.test("jpeg.wasm: undecodable bytes return -2", async () => {
  const { exports } = await instantiate("jpeg.wasm");
  const truncated = new Uint8Array([0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01]);
  const result = decode(exports, truncated);
  assert(result.status === -2, `expected -2, got ${result.status}`);
});

Deno.test("webp.wasm: an opaque lossless WebP widens RGB to RGBA with alpha 255", async () => {
  const { exports } = await instantiate("webp.wasm");
  const result = decode(exports, await fixture("test-marker-lossless.webp"));
  assert(result.status === 0, `decode returned ${result.status}`);
  assert(result.width === 8 && result.height === 8, `expected 8x8, got ${result.width}x${result.height}`);
  const alphas = alphaValues(result.pixels);
  assert(alphas.size === 1 && alphas.has(255), `expected alpha 255 only, got [${[...alphas]}]`);
});

Deno.test("webp.wasm: alpha survives decode", async () => {
  const { exports } = await instantiate("webp.wasm");
  const result = decode(exports, await fixture("test-alpha-lossless.webp"));
  assert(result.status === 0, `decode returned ${result.status}`);
  // The fixture has transparent, half and opaque regions.
  const alphas = alphaValues(result.pixels);
  assert(alphas.size === 3, `expected three alpha values, got [${[...alphas].sort((a, b) => a - b)}]`);
});

Deno.test("webp.wasm: a lossy WebP decodes to RGBA8 at its intrinsic size", async () => {
  const { exports } = await instantiate("webp.wasm");
  const result = decode(exports, await fixture("test-photo-lossy.webp"));
  assert(result.status === 0, `decode returned ${result.status}`);
  assert(result.width === 360 && result.height === 540, `expected 360x540, got ${result.width}x${result.height}`);
});

Deno.test("webp.wasm: an animated WebP returns -4 and reports its frame count", async () => {
  const { exports } = await instantiate("webp.wasm");
  const result = decode(exports, await fixture("test-anim.webp"));
  assert(result.status === -4, `expected -4, got ${result.status}`);
  assert(result.frameCount === 3, `expected 3 frames, got ${result.frameCount}`);
});

Deno.test("webp.wasm: undecodable bytes return -2", async () => {
  const { exports } = await instantiate("webp.wasm");
  const header = [0x52, 0x49, 0x46, 0x46, 0x20, 0, 0, 0, 0x57, 0x45, 0x42, 0x50];
  const result = decode(exports, new Uint8Array([...header, ...new Array(20).fill(0x7F)]));
  assert(result.status === -2, `expected -2, got ${result.status}`);
});
