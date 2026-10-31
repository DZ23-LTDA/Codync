import assert from "node:assert/strict";
import { test } from "node:test";

import { enUS, ptBR } from "./catalog.ts";
import { isLocale, locale, setLocale, t } from "./index.ts";

test("every English key has a Portuguese translation", () => {
  const missing = Object.keys(enUS).filter((key) => !(key in ptBR));
  assert.deepEqual(missing, [], "a key without pt-BR would silently read in English");
});

test("Portuguese carries no key English doesn't have", () => {
  const orphans = Object.keys(ptBR).filter((key) => !(key in enUS));
  assert.deepEqual(orphans, [], "an orphan key is a translation of something that no longer exists");
});

test("the same message reads differently in each language", () => {
  setLocale("en-US");
  assert.equal(t("common.cancel"), "Cancel");
  setLocale("pt-BR");
  assert.equal(t("common.cancel"), "Cancelar");
  setLocale("en-US");
});

test("placeholders are filled and unknown ones are left alone", () => {
  setLocale("en-US");
  assert.equal(t("bot.missing", { name: "research" }), "Bot research not found.");
  setLocale("pt-BR");
  assert.equal(t("bot.missing", { name: "pesquisa" }), "Bot pesquisa não encontrado.");
  setLocale("en-US");
  assert.equal(t("bot.missing"), "Bot {name} not found.", "an unfilled placeholder stays visible, not empty");
});

test("only the languages this build carries are accepted", () => {
  assert.ok(isLocale("pt-BR"));
  assert.ok(isLocale("en-US"));
  assert.ok(!isLocale("de-DE"));
  assert.ok(!isLocale("toString"), "an inherited property is not a language");
});

test("the locale starts in English and follows the setting", () => {
  setLocale("en-US");
  assert.equal(locale(), "en-US");
  setLocale("pt-BR");
  assert.equal(locale(), "pt-BR");
  setLocale("en-US");
});
