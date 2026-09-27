import { execSync } from "node:child_process";

const task = process.argv[2] ?? "build";
const pkg =
  process.platform === "darwin" ? "vision-api-macos" : "vision-api-linux";

execSync(`turbo run ${task} --filter=${pkg}`, {
  stdio: "inherit",
  env: process.env,
});
