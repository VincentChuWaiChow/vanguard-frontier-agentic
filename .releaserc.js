"use strict";

const SESSION_URL_RE = /https?:\/\/claude\.ai\/code\/session_\S+/g;

function cleanBody(body) {
  if (!body) return body;
  return body
    .replace(SESSION_URL_RE, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim() || undefined;
}

// Conventional-commit type -> human section. A custom writerOpts.transform
// replaces the preset's own transform, so we reproduce its responsibilities
// here: map types to sections, drop hidden types, set the short hash, and
// normalize the breaking-change note title.
const RELEASE_SECTIONS = {
  feat: "✨ Features",
  fix: "🐛 Bug Fixes",
  security: "🔒 Security",
  perf: "⚡ Performance",
  refactor: "♻️ Refactor",
  docs: "📚 Documentation",
  build: "📦 Build & Dependencies",
  revert: "⏪ Reverts",
};

// Render order for the grouped sections (breaking changes are note groups and
// always render first via the preset's main template).
const SECTION_ORDER = [
  ...Object.values(RELEASE_SECTIONS),
  "🔧 Other Changes",
];

module.exports = {
  branches: [
    "master",
    { name: "develop", prerelease: "alpha" },
  ],
  plugins: [
    [
      "@semantic-release/commit-analyzer",
      {
        preset: "conventionalcommits",
        releaseRules: [
          { type: "security", release: "patch" },
          { type: "perf", release: "patch" },
          { type: "refactor", release: "patch" },
          { type: "build", release: "patch" },
          { type: "revert", release: "patch" },
          { scope: "no-release", release: false },
        ],
      },
    ],
    [
      "@semantic-release/release-notes-generator",
      {
        preset: "conventionalcommits",
        presetConfig: {
          types: [
            { type: "feat", section: "✨ Features" },
            { type: "fix", section: "🐛 Bug Fixes" },
            { type: "security", section: "🔒 Security" },
            { type: "perf", section: "⚡ Performance" },
            { type: "refactor", section: "♻️ Refactor" },
            { type: "docs", section: "📚 Documentation" },
            { type: "build", section: "📦 Build & Dependencies" },
            { type: "revert", section: "⏪ Reverts" },
            { type: "test", hidden: true },
            { type: "ci", hidden: true },
            { type: "chore", hidden: true },
            { type: "style", hidden: true },
          ],
        },
        // conventional-changelog-writer@9 (required by the conventionalcommits
        // preset from v10) takes render functions, not Handlebars strings. The
        // preset's main template joins header, note groups, commit groups and
        // footer with blank lines, and its list() adds the "* " bullet and
        // indents a commit's body lines beneath it — so none of these partials
        // carries its own bullet or surrounding blank lines.
        writerOpts: {
          // Enterprise-style header: versioned title + release-type banner.
          // The conventionalcommits preset renders the "⚠ BREAKING CHANGES"
          // block immediately after this, so a major release leads with the
          // migration-critical information.
          headerPartial: (context) =>
            `## 🛡️ v${context.version} — *Provenance · Policy · Portability*\n` +
            `${context.date ? `_Released ${context.date}_` : ""}\n\n` +
            "> _Curated multi-cloud, zero-trust agent marketplace — `AWS` · `Azure` · `OCI` · `GCP` · `Terraform`._\n" +
            "> Least privilege, live evidence, safe rollback paths.\n\n" +
            (context.isPatch
              ? "**Release type:** Maintenance & hardening."
              : "**Release type:** New capabilities — review the sections below before upgrading."),
          // Deterministic commit line with a linked short hash.
          commitPartial: (context, commit) =>
            `${commit.scope ? `**${commit.scope}:** ` : ""}${commit.subject || ""}` +
            (commit.shortHash
              ? ` ([\`${commit.shortHash}\`](${
                  context.host
                    ? `${context.host}/${context.owner}/${context.repository}/commit/${commit.hash}`
                    : "#"
                }))`
              : "") +
            (commit.body ? `\n${commit.body}` : ""),
          // Enterprise footer: install path, supply-chain provenance, and a
          // deterministic full-changelog compare link.
          footerPartial: (context) =>
            "---\n\n" +
            "### 📥 Install\n" +
            `\`\`\`bash\nnpm install @raishin/vanguard-frontier-agentic@${context.version}\n\`\`\`\n\n` +
            "### 🔐 Supply-chain provenance\n" +
            "Every release ships a build attestation (SLSA provenance) and an SBOM. " +
            "Verify the tag with `gh attestation verify` before installing." +
            (context.previousTag
              ? "\n\n**Full changelog:** " +
                `${context.host}/${context.owner}/${context.repository}/compare/${context.previousTag}...${context.currentTag}`
              : ""),
          // Deterministic section ordering (note groups / breaking changes
          // render first regardless).
          commitGroupsSort(a, b) {
            return SECTION_ORDER.indexOf(a.title) - SECTION_ORDER.indexOf(b.title);
          },
          commitsSort: ["scope", "subject"],
          // conventional-changelog-writer v9 passes a read-only commit and
          // merges the returned object over it as a patch, so we return only
          // the fields we change. We also assume the preset's transform
          // duties: map type -> section, drop hidden types (unless they carry
          // a breaking-change note), set the short hash, and title the note
          // group. Returning a falsy value drops the commit from the notes.
          transform(commit) {
            // Normalize the note title to the plural form; the preset's main
            // template already prepends the "⚠" warning sign to the heading.
            const notes = commit.notes.map((note) => ({
              ...note,
              title: "BREAKING CHANGES",
            }));
            const section = RELEASE_SECTIONS[commit.type];
            if (!section && notes.length === 0) return false;
            return {
              notes,
              type: section || "🔧 Other Changes",
              shortHash:
                typeof commit.hash === "string"
                  ? commit.hash.substring(0, 7)
                  : commit.shortHash,
              body: cleanBody(commit.body),
            };
          },
        },
      },
    ],
    ["@semantic-release/changelog", { changelogFile: "CHANGELOG.md" }],
    [
      "@semantic-release/exec",
      {
        // Runs AFTER changelog writes and BEFORE npm packs the tarball.
        // Synchronizes plugin manifest versions and the cross-asset
        // integrity manifest to the bumped package.json so the released
        // tarball is internally consistent. See scripts/release-prepare.mjs.
        prepareCmd: "node scripts/release-prepare.mjs ${nextRelease.version}",
      },
    ],
    [
      "@semantic-release/npm",
      {
        // OIDC trusted publishing: when ACTIONS_ID_TOKEN_REQUEST_URL is set
        // (GitHub Actions with id-token: write), npm CLI automatically performs
        // OIDC token exchange without requiring NPM_TOKEN.
        // Disable npm plugin's publish and verifyConditions steps (which require
        // a token). We handle publishing separately via 'npm publish' in the
        // workflow after semantic-release completes versioning.
        npmPublish: false,
      },
    ],
    [
      "@semantic-release/github",
      {
        successComment: false,
        failComment: false,
        // Custom, branded GitHub Release title (lodash template). Defaults to
        // the tag (`v3.0.0`); we lead with the shield + theme for a polished
        // release page while keeping the version front-and-center.
        releaseNameTemplate:
          "🛡️ v<%= nextRelease.version %> — Provenance · Policy · Portability",
        // Link prior releases on the release page and label shipped work.
        addReleases: "bottom",
        releasedLabels: ["released"],
      },
    ],
    [
      "@semantic-release/git",
      {
        assets: [
          "CHANGELOG.md",
          "package.json",
          // Synchronized by scripts/release-prepare.mjs during prepare.
          // package.json is the single source of truth; all versioned files
          // below are derived from it and committed together in the release
          // commit so the next diff stays clean and the attested tarball
          // matches the committed tree.
          ".claude-plugin/plugin.json",
          ".claude-plugin/marketplace.json",
          ".cursor-plugin/plugin.json",
          "plugins/vanguard-frontier-agentic/.codex-plugin/plugin.json",
          ".github/plugin/marketplace.json",
          "SECURITY.md",
          "catalog/asset-integrity.json",
        ],
        message: "chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}",
      },
    ],
  ],
};
