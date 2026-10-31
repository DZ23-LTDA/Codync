import assert from "node:assert/strict";
import { test } from "node:test";

import { appLanguages, languageTitle, resolvedLanguage, translate } from "./l10n.ts";

test("Brazilian Portuguese is a language this app offers", () => {
  assert.ok(appLanguages.includes("pt"));
  assert.equal(resolvedLanguage("pt", "en-US"), "pt");
  assert.equal(languageTitle("pt", "pt", "en-US"), "Português (Brasil)");
});

test("a system set to Portuguese lands on it, in any regional shape", () => {
  assert.equal(resolvedLanguage("system", "pt-BR"), "pt");
  assert.equal(resolvedLanguage("system", "pt"), "pt");
  assert.equal(resolvedLanguage("system", "pt_PT"), "pt");
});

test("a phrase the Portuguese table carries comes back in Portuguese", () => {
  assert.equal(translate("Something went wrong", "pt", "en-US"), "Algo deu errado");
  assert.equal(translate("No bots yet", "pt", "en-US"), "Nenhum bot ainda");
});

test("a phrase it does not carry stays in English instead of turning into a key", () => {
  assert.equal(translate("A phrase no table knows", "pt", "en-US"), "A phrase no table knows");
});

test("placeholders survive the translation", () => {
  assert.equal(translate("{done} of {total} ready", "pt", "en-US"), "{done} de {total} prontos");
});
