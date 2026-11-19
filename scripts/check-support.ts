import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, posix, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  type DocumentManifest,
  documentExitCode,
  parseDocumentManifest,
  parseDocumentReport,
  prepareDocuments,
  scanDocuments,
} from "jev-lint/documents";

interface Evidence {
  readonly path: string;
  readonly topic: string;
  readonly contains?: string;
  readonly targetContains?: string;
  readonly context?: readonly string[];
}
interface CrateScope {
  readonly name: string;
  readonly evidence: readonly Evidence[];
}
export interface SourceView {
  readonly paths: readonly string[];
  readonly read: (path: string) => Promise<Buffer>;
}

const scopes: readonly CrateScope[] = [
  {
    name: "rtla",
    evidence: [
      {
        path: "src/lib.rs",
        topic: "公開parser API、構文と評価の分離、整数と深さの上限",
      },
      {
        path: "src/parser.rs",
        topic: "対応する構文と未対応構文、infixとCASEの解析",
      },
    ],
  },
  {
    name: "rtla-compiler",
    evidence: [
      {
        path: "src/link.rs",
        topic:
          "明示したmodule集合のEXTENDS/INSTANCE、LOCALとscope、設定置換の対応範囲・拒否と展開上限",
        context: ["src/rewrite.rs"],
      },
      {
        path: "src/lib.rs",
        contains: "pub fn compile(",
        topic: "受理するmodule/configと、実行しない・拒否する構文の境界",
      },
      {
        path: "src/lib.rs",
        contains: "fn apply(",
        targetContains: "signed 32-bit整数",
        topic:
          "有限式、action、CASE、Apalache foldsと限定したVariantsのlowering",
      },
    ],
  },
  {
    name: "rtlc",
    evidence: [
      {
        path: "src/ir.rs",
        topic: "IRの値と式・予算・結果の公開契約",
        context: ["src/value.rs", "src/safety.rs"],
      },
      {
        path: "src/safety.rs",
        topic: "有限BFS、invariantとdeadlock、結果と探索予算",
      },
      {
        path: "src/temporal.rs",
        topic: "temporal/fairnessの検査とlasso、保証範囲の区別",
      },
      {
        path: "src/safety_parallel.rs",
        topic: "native内部並列化の対象、逐次fallback、workerとworkの契約",
      },
    ],
  },
  {
    name: "idealis-core",
    evidence: [
      {
        path: "src/lib.rs",
        topic: "Model/Driver effect、照合protocol、TLA検査への委譲",
      },
      {
        path: "src/history.rs",
        topic: "並行履歴の確定・未確定イベントと有限探索予算",
      },
      {
        path: "src/host.rs",
        topic: "明示した時計・通信・永続化とHostTapeでありOS障害ではないこと",
      },
    ],
  },
  {
    name: "idealis-smt",
    evidence: [
      {
        path: "src/parser.rs",
        topic: "受理するSMT-LIB式・型・制約と拒否する範囲",
      },
      {
        path: "src/solver.rs",
        topic: "assertion scope、一時assumptions、求解結果の契約",
      },
      {
        path: "src/script.rs",
        topic: "対応scriptコマンド、checkの順序、共有workと空の結果列",
      },
    ],
  },
  {
    name: "ralloy",
    evidence: [
      {
        path: "src/parser.rs",
        topic: "静的なAlloyの受理する構文と有限scope、未対応構文",
        context: ["src/syntax.rs", "src/eval.rs", "src/lib.rs"],
      },
      {
        path: "src/lib.rs",
        topic: "公開APIと結果status、解析・固定instance検査の区別",
      },
      {
        path: "src/check.rs",
        topic: "固定した実装instanceを検査し、invalidやunknownを区別する契約",
        context: ["src/instance.rs", "src/syntax.rs"],
      },
    ],
  },
  {
    name: "idealis-testing",
    evidence: [
      {
        path: "src/session.rs",
        topic: "PBT effect、失敗signature、縮小と再生、未確定と成功の区別",
      },
      {
        path: "src/shrink.rs",
        topic: "操作列の削除による縮小でありscalarの縮小ではないこと",
      },
      {
        path: "src/schedule.rs",
        topic: "明示したactor操作順と構造上限、自動thread計装ではないこと",
      },
      {
        path: "src/compare.rs",
        topic: "許容観測集合との比較とerror/unknown/invalidの保持",
      },
    ],
  },
];
const shared = ["scripts/check-support.ts", "package.json", "pnpm-lock.yaml"];
const manifestPath = (name: string) =>
  `.jev-lint/checklists/${name}.manifest.json`;
const recordPath = (name: string) => `.jev-lint/checklists/${name}.record.json`;
const git = (root: string, args: readonly string[]) =>
  execFileSync("git", [...args], { cwd: root, maxBuffer: 16 * 1024 * 1024 });

export const sourceView = (root: string, staged: boolean): SourceView => {
  if (staged) {
    const entries = new Map(
      git(root, ["ls-files", "--stage", "-z"])
        .toString()
        .split("\0")
        .filter(Boolean)
        .map((entry) => {
          const match = /^(\d+) ([a-f0-9]+) (\d)\t([\s\S]+)$/.exec(entry);
          assert.ok(
            match && match[3] === "0",
            "Unmerged index entries cannot be reviewed",
          );
          return [
            match[4] ?? "",
            { mode: match[1], oid: match[2] ?? "" },
          ] as const;
        }),
    );
    return {
      paths: [...entries.keys()].sort(),
      read: async (path) => {
        const entry = entries.get(path);
        assert.ok(
          entry && ["100644", "100755"].includes(entry.mode ?? ""),
          `Missing or non-regular staged source: ${path}`,
        );
        return git(root, ["cat-file", "blob", entry.oid]);
      },
    };
  }
  return {
    paths: git(root, [
      "ls-files",
      "-z",
      "--cached",
      "--others",
      "--exclude-standard",
    ])
      .toString()
      .split("\0")
      .filter(Boolean)
      .sort(),
    read: async (path) => {
      assert.ok(!posix.isAbsolute(path) && !path.split("/").includes(".."));
      for (const part of path
        .split("/")
        .map((_, index, parts) => parts.slice(0, index + 1).join("/")))
        assert.ok(
          !(await lstat(join(root, part))).isSymbolicLink(),
          `Symlink source: ${path}`,
        );
      assert.ok(
        (await lstat(join(root, path))).isFile(),
        `Not a regular file: ${path}`,
      );
      return readFile(join(root, path));
    },
  };
};

export const checkSupport = async (
  view: SourceView,
  crateNames: readonly string[],
): Promise<void> => {
  const actual = view.paths
    .flatMap(
      (path) => /^crates\/([^/]+)\/Cargo.toml$/.exec(path)?.slice(1) ?? [],
    )
    .sort();
  assert.deepEqual(
    actual,
    [...crateNames].sort(),
    "Crate inventory changed; update the checklist catalog",
  );
  for (const name of actual) {
    const path = `crates/${name}/SUPPORT.md`;
    const content = (await view.read(path)).toString();
    const visible: string[] = [];
    let fence: { marker: string; length: number } | undefined;
    for (const line of content.split("\n")) {
      const marker = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
      if (marker) {
        if (!fence) fence = { marker: marker[0] ?? "", length: marker.length };
        else if (marker[0] === fence.marker && marker.length >= fence.length)
          fence = undefined;
        continue;
      }
      if (!fence) visible.push(line);
    }
    assert.match(
      visible.join("\n"),
      /^\s*[-*] \[[xX]\] .+/m,
      `${path}: supported checklist is missing`,
    );
    assert.match(
      visible.join("\n"),
      /^\s*[-*] \[ \] .+/m,
      `${path}: unsupported checklist is missing`,
    );
    for (const link of content.matchAll(/\]\(([^)]+)\)/g)) {
      const target = link[1] ?? "";
      if (/^(?:https?:|#)/.test(target)) continue;
      const destination = posix.normalize(
        posix.join(posix.dirname(path), target.split("#")[0] ?? ""),
      );
      assert.ok(
        view.paths.includes(destination),
        `${path}: link absent from selected tree: ${target}`,
      );
      await view.read(destination);
    }
  }
};

const manifestFor = (view: SourceView, scope: CrateScope): DocumentManifest => {
  const prefix = `crates/${scope.name}/`;
  const paths = [
    ...new Set([
      ...shared,
      ...view.paths.filter(
        (path) =>
          path.startsWith(prefix) &&
          (/\.(?:rs|toml|lock|json|tla|cfg)$/.test(path) ||
            ["README.md", "SUPPORT.md"].includes(path.slice(prefix.length))),
      ),
    ]),
  ].sort();
  const files = Object.fromEntries(
    paths.map((path, index) => [
      `source${index}`,
      { path, role: path === `${prefix}SUPPORT.md` ? "document" : "reference" },
    ]),
  );
  const idFor = (path: string) => {
    const entry = Object.entries(files).find(([, file]) => file.path === path);
    assert.ok(entry, `Unregistered support source: ${path}`);
    return entry[0];
  };
  return parseDocumentManifest({
    schema: "jev-lint-documents-manifest-1",
    roots: [`${prefix}SUPPORT.md`],
    files,
    comparisons: [
      {
        id: `${scope.name}-readme`,
        topic: "記載条件内の対応機能、未対応・対象外、数値や結果の制約",
        mode: "semantic",
        reference: { source: idFor(`${prefix}README.md`) },
        target: { source: idFor(`${prefix}SUPPORT.md`) },
      },
      ...scope.evidence.map((evidence, index) => ({
        id: `${scope.name}-implementation-${index}`,
        topic: evidence.topic,
        mode: "semantic",
        reference: {
          source: idFor(prefix + evidence.path),
          ...(evidence.contains ? { contains: evidence.contains } : {}),
        },
        target: {
          source: idFor(`${prefix}SUPPORT.md`),
          ...(evidence.targetContains
            ? { contains: evidence.targetContains }
            : {}),
        },
        context: (evidence.context ?? []).map((path) => ({
          source: idFor(prefix + path),
        })),
      })),
    ],
  });
};

const snapshot = async (
  view: SourceView,
  manifest: DocumentManifest,
  directory: string,
) => {
  for (const source of Object.values(manifest.files)) {
    await mkdir(dirname(join(directory, source.path)), { recursive: true });
    await writeFile(join(directory, source.path), await view.read(source.path));
  }
  const input = prepareDocuments(manifest, { cwd: directory });
  assert.deepEqual(input.issues, [], JSON.stringify(input.issues));
  assert.equal(input.deferred.length, 0);
  assert.equal(input.unboundDocuments.length, 0);
  assert.equal(input.comparisons.length, input.expectedComparisons);
  return input;
};

export const installSupportHook = async (root: string): Promise<void> => {
  const path = resolve(
    root,
    git(root, ["rev-parse", "--git-path", "hooks/pre-commit"])
      .toString()
      .trim(),
  );
  const shim =
    '#!/bin/sh\nhook="$(git rev-parse --show-toplevel)/.jev-lint/hooks/pre-commit" || exit 2\nif [ -x "$hook" ]; then exec "$hook" "$@"; fi\n';
  await mkdir(dirname(path), { recursive: true });
  const present = await lstat(path).catch((error: NodeJS.ErrnoException) => {
    if (error.code !== "ENOENT") throw error;
    return null;
  });
  if (present)
    assert.ok(
      present.isFile() && (await readFile(path, "utf8")) === shim,
      "Existing pre-commit hook kept; invoke .jev-lint/hooks/pre-commit from it",
    );
  else await writeFile(path, shim, { flag: "wx" });
  await chmod(path, 0o755);
};

const run = async () => {
  const args = process.argv.slice(2);
  assert.ok(
    args.every((arg) =>
      ["--staged", "--sync", "--dry-run", "--install-hook"].includes(arg),
    ),
    "Unknown support-check option",
  );
  const sync = args.includes("--sync");
  assert.ok(
    !sync || !args.includes("--dry-run"),
    "Use dry-run before sync, not together",
  );
  if (
    sync &&
    !args.includes("--install-hook") &&
    !process.env.TYPESAFE_API_KEY &&
    !process.env.TYPESAFEAI_API_KEY
  ) {
    console.warn(
      "warning: Jev API key is not set (TYPESAFE_API_KEY / TYPESAFEAI_API_KEY); skipping live checklist review.",
    );
    return;
  }
  const root = git(process.cwd(), ["rev-parse", "--show-toplevel"])
    .toString()
    .trim();
  if (args.includes("--install-hook")) {
    await installSupportHook(root);
    return;
  }
  const staged = args.includes("--staged");
  const view = sourceView(root, staged);
  await checkSupport(
    view,
    scopes.map((scope) => scope.name),
  );
  let code = 0;
  for (const scope of scopes) {
    const directory = await mkdtemp(join(tmpdir(), "idealis-support-"));
    try {
      const manifest = manifestFor(view, scope);
      const input = await snapshot(view, manifest, directory);
      if (args.includes("--dry-run")) {
        console.log(
          `${scope.name}: ${input.comparisons.length} comparisons, ${input.sources.length} sources, ${input.comparisons.reduce((sum, item) => sum + JSON.stringify(item).length, 0)} chars`,
        );
        continue;
      }
      if (sync) {
        await mkdir(join(root, ".jev-lint/checklists"), { recursive: true });
        await writeFile(
          join(root, manifestPath(scope.name)),
          `${JSON.stringify(manifest, null, 2)}\n`,
        );
        const report = await scanDocuments(input, {
          clientOptions: { model: "jev-1.14.0" },
          snapshotCheck: async () => {
            const current = sourceView(root, staged);
            assert.deepEqual(manifestFor(current, scope), manifest);
            for (const source of Object.values(manifest.files))
              assert.deepEqual(
                await current.read(source.path),
                await readFile(join(directory, source.path)),
              );
            return true;
          },
        });
        await writeFile(
          join(root, recordPath(scope.name)),
          `${JSON.stringify(report, null, 2)}\n`,
        );
        const exit = documentExitCode(report);
        console.log(
          `${scope.name}: ${report.status}, ${report.coverage.contradictions} contradictions, ${report.coverage.unassessed} unassessed, $${report.usage.usd.toFixed(6)}`,
        );
        for (const candidate of report.candidates)
          if (candidate.judgment?.choice !== "supports")
            console.log(
              `${candidate.id}: ${candidate.judgment?.choice ?? "unanswered"}`,
            );
        if (exit) code = exit;
      } else {
        const savedManifest: unknown = JSON.parse(
          (await view.read(manifestPath(scope.name))).toString(),
        );
        assert.deepEqual(
          savedManifest,
          manifest,
          "Stale checklist manifest; run just support-sync and stage its outputs",
        );
        const report = parseDocumentReport(
          JSON.parse((await view.read(recordPath(scope.name))).toString()),
          input,
        );
        const exit = documentExitCode(report);
        console.log(
          `${scope.name}: ${report.status}, ${report.coverage.answered}/${report.coverage.expected} answered (offline)`,
        );
        if (exit) code = exit;
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  }
  if (sync && code === 0)
    execFileSync(
      "pnpm",
      ["exec", "biome", "format", "--write", ".jev-lint/checklists"],
      { cwd: root, stdio: "inherit" },
    );
  process.exitCode = code;
};

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  await run().catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : String(error));
    console.error(
      "Checklist review is incomplete. Run just support-sync (or support-sync-staged), inspect the findings, then stage SUPPORT.md and .jev-lint/checklists.",
    );
    process.exitCode = 3;
  });
