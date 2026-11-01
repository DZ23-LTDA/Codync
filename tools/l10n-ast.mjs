#!/usr/bin/env node
// Finds copy in the desktop renderer that never reaches `tr()`, by reading the syntax tree instead of the text.
//
// Why an AST: the regex scanners read literals and JSX expressions, so a template literal -- `Open ${bot.name}'s chat`,
// `Computers, ${text}. Filter conversations` -- was invisible to all four of them, and five such strings shipped in
// English. The compiler is already a dependency of the renderer, so the honest scanner uses it.
//
// Usage: node tools/l10n-ast.mjs [--list]
import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

const ts = (await import("../apps/desktop/node_modules/typescript/lib/typescript.js")).default;
const ROOT = "apps/desktop/src/renderer";
const LIST = process.argv.includes("--list");

/** Attributes whose value is read by the person, directly or through a screen reader. */
const COPY_ATTRIBUTES = new Set(["title", "aria-label", "placeholder", "alt", "footer", "detail", "empty", "label", "message"]);

/** Text that is never translated: keys, paths, code, addresses, and the demo's own fixture data. */
const NOT_COPY = [
  /^https?:\/\//,
  /^[a-z0-9-]+(\.[a-z0-9-]+)+$/i,
  /^[A-Z_]+$/,
  /^[\d\s.,:/-]+$/,
  /^\{\{.*\}\}$/,
  /^--/,
];

const words = (text) => text.trim().split(/\s+/).filter((part) => /[A-Za-zÀ-ÿ]{2,}/.test(part)).length;
const isCopy = (text) => {
  const trimmed = text.trim();
  if (trimmed.length < 3 || !/[A-Za-zÀ-ÿ]/.test(trimmed)) return false;
  if (NOT_COPY.some((pattern) => pattern.test(trimmed))) return false;
  return words(trimmed) >= 1;
};

function walkFiles(dir) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "demo") continue;
      found.push(...walkFiles(full));
    } else if (entry.name.endsWith(".tsx") && !entry.name.includes(".test.")) {
      found.push(full);
    }
  }
  return found;
}


/** The words a template puts on screen by itself, without what the pieces add. */
const literalPart = (template) =>
  [template.head.text, ...template.templateSpans.map((span) => span.literal.text)].join(" ");

/** True when the expression sits between the tags of an element, where its value is read. */
function isJsxChild(node) {
  const parent = node.parent;
  return Boolean(parent && (ts.isJsxElement(parent) || ts.isJsxFragment(parent)) && parent.children.includes(node));
}

const findings = [];

function report(node, file, text, kind) {
  const { line } = node.getSourceFile().getLineAndCharacterOfPosition(node.getStart());
  findings.push({ file, line: line + 1, kind, text: text.replace(/\s+/g, " ").trim() });
}

function scan(file) {
  const source = readFileSync(file, "utf8");
  if (source.includes("@l10n-ast-ignore-file")) return;
  const sf = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);

  const visit = (node, inTr) => {
    // `tr(...)` is the translation itself: whatever is inside it is already handled.
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === "tr") {
      node.arguments.forEach((argument) => visit(argument, true));
      return;
    }
    if (inTr) {
      ts.forEachChild(node, (child) => visit(child, true));
      return;
    }

    if (ts.isJsxText(node) && isCopy(node.text)) report(node, file, node.text, "jsx-text");

    if (ts.isJsxAttribute(node) && COPY_ATTRIBUTES.has(node.name.getText(sf))) {
      const init = node.initializer;
      if (init && ts.isStringLiteral(init) && isCopy(init.text)) report(init, file, init.text, "attribute");
    }

    // A template literal only counts where a person reads it: as a JSX child or in a copy attribute. In `className`,
    // `style`, `key` or a function argument it is a class name or a number, and counting those buried the real ones.
    const asChild = isJsxChild(node);
    if (ts.isJsxExpression(node) && node.expression && asChild) {
      const expression = node.expression;
      if (ts.isStringLiteral(expression) && isCopy(expression.text)) report(expression, file, expression.text, "jsx-expression");
      if (ts.isTemplateExpression(expression) && isCopy(literalPart(expression))) {
        report(expression, file, expression.getText(sf), "template");
      }
    }

    if (
      ts.isJsxAttribute(node) &&
      COPY_ATTRIBUTES.has(node.name.getText(sf)) &&
      node.initializer &&
      ts.isJsxExpression(node.initializer) &&
      node.initializer.expression &&
      ts.isTemplateExpression(node.initializer.expression) &&
      isCopy(literalPart(node.initializer.expression))
    ) {
      report(node.initializer.expression, file, node.initializer.expression.getText(sf), "attribute-template");
    }

    ts.forEachChild(node, (child) => visit(child, false));
  };

  visit(sf, false);
}

for (const file of walkFiles(ROOT)) scan(file);

const byKind = findings.reduce((tally, finding) => ({ ...tally, [finding.kind]: (tally[finding.kind] ?? 0) + 1 }), {});
console.log(`${findings.length} visible strings never reach tr() in ${ROOT}:`);
for (const [kind, count] of Object.entries(byKind).sort((a, b) => b[1] - a[1])) console.log(`  ${String(count).padStart(3)}  ${kind}`);

if (LIST) {
  const byFile = new Map();
  for (const finding of findings) {
    const key = relative(".", finding.file).split(sep).join("/");
    byFile.set(key, [...(byFile.get(key) ?? []), finding]);
  }
  for (const [file, list] of byFile) {
    console.log(`\n${file}`);
    for (const finding of list) console.log(`  ${finding.line}: ${finding.text.slice(0, 100)}`);
  }
}

process.exit(findings.length === 0 ? 0 : 1);
