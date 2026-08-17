import { readFile, writeFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const requested = process.argv[2];
const checkOnly = requested === "--check";
const packageJson = JSON.parse(await readFile(new URL("package.json", root), "utf8"));
const version = checkOnly ? packageJson.version : requested;

if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Usage: npm run version:set -- <semver>");
  process.exit(1);
}

const jsonFiles = ["package.json", "package-lock.json", "src-tauri/tauri.conf.json"];
const observed = [];
for (const file of jsonFiles) {
  const url = new URL(file, root);
  const value = JSON.parse(await readFile(url, "utf8"));
  observed.push([file, value.version]);
  if (file === "package-lock.json") observed.push([`${file} (root package)`, value.packages?.[""]?.version]);
  if (!checkOnly) {
    value.version = version;
    if (file === "package-lock.json") value.packages[""].version = version;
    await writeFile(url, `${JSON.stringify(value, null, 2)}\n`);
  }
}

const cargoUrl = new URL("src-tauri/Cargo.toml", root);
const cargo = await readFile(cargoUrl, "utf8");
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1];
observed.push(["src-tauri/Cargo.toml", cargoVersion]);
if (!checkOnly) await writeFile(cargoUrl, cargo.replace(/^version = "[^"]+"/m, `version = "${version}"`));

const cargoLockUrl = new URL("src-tauri/Cargo.lock", root);
const cargoLock = await readFile(cargoLockUrl, "utf8");
const graftPackage = /(\[\[package\]\]\nname = "graft"\nversion = ")([^"]+)(")/;
const cargoLockVersion = cargoLock.match(graftPackage)?.[2];
observed.push(["src-tauri/Cargo.lock (graft package)", cargoLockVersion]);
if (!checkOnly) await writeFile(cargoLockUrl, cargoLock.replace(graftPackage, `$1${version}$3`));

if (checkOnly) {
  const mismatches = observed.filter(([, current]) => current !== version);
  if (mismatches.length) {
    for (const [file, current] of mismatches) console.error(`${file}: expected ${version}, found ${current ?? "missing"}`);
    process.exit(1);
  }
  console.log(`All manifests use ${version}`);
} else {
  console.log(`Set Graft version to ${version}`);
}
