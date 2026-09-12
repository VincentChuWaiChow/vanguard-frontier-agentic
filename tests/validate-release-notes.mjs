import { createRequire } from "node:module";
import { generateNotes } from "@semantic-release/release-notes-generator";

const require = createRequire(import.meta.url);
const config = require("../.releaserc.js");
const pluginConfig = config.plugins.find(
  ([name]) => name === "@semantic-release/release-notes-generator",
)?.[1];

if (!pluginConfig) {
  throw new Error("release-notes-generator configuration is missing");
}

const notes = await generateNotes(pluginConfig, {
  commits: [
    {
      hash: "1234567890abcdef",
      message: "feat(istio): add evidence-bounded review suite",
    },
  ],
  lastRelease: { gitTag: "v3.11.1", gitHead: "old" },
  nextRelease: { version: "3.12.0", gitTag: "v3.12.0", gitHead: "new" },
  options: {
    repositoryUrl:
      "https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git",
  },
  cwd: process.cwd(),
  logger: console,
});

if (!notes.includes("v3.12.0") || !notes.includes("evidence-bounded review suite")) {
  throw new Error("release notes are missing the expected version or feature");
}

console.log("OK: semantic-release notes generated with the configured preset");
