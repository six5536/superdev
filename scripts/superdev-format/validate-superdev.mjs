#!/usr/bin/env node
// validate-awa.mjs — structural lint for awa-format conversions.
//
// Usage: node validate-awa.mjs [--grammar <file>] [--meta <file>]
//                              [--kind unit|schema|core] <file...>
//        node validate-awa.mjs --doc
//
// Every structural rule this tool enforces is declared in superdev-grammar.yaml:
// the element vocabulary, their attributes, occurrence and nesting, the one
// condition attribute, the rule levels, the ledger dispositions, and the
// duplication tuning. Change the language there, not here. `--doc` prints the
// grammar as a markdown reference.
//
// The grammar is itself checked against superdev-grammar.meta.yaml before use (a
// sibling of the grammar file by default, or --meta), so a typo in the grammar
// fails loudly instead of silently switching a rule off.
//
// Kinds are matched by filename per the grammar's `match` blocks, and may be
// overridden with --kind, applied to every listed file.
//
// Pass every file of a conversion in ONE invocation: the duplication check
// compares statements within each unit and across the invocation's units,
// core, and schemas.
//
// Exit code: 0 all files pass, 1 any error, 2 usage/internal error.

import { readFileSync, existsSync, readdirSync } from 'node:fs';
import YAML from 'yaml';
import { basename, dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
// The grammar's home is the format capability's directory, beside the
// instruction files, the way `.agents/aokf/SPEC.md` sits beside aokf.md.
const DEFAULT_GRAMMAR = join(HERE, '..', '..', '.agents', 'format', 'grammar.yaml');

// ---------- generic helpers ----------

function fenceMap(ls) {
  // true for every line inside (or delimiting) a fenced code block.
  const map = new Array(ls.length).fill(false);
  let fence = null; // { char, len }
  for (let i = 0; i < ls.length; i++) {
    const m = ls[i].match(/^\s*(`{3,}|~{3,})/);
    if (fence) {
      map[i] = true;
      if (m && m[1][0] === fence.char && m[1].length >= fence.len && ls[i].trim() === m[1]) {
        fence = null;
      }
    } else if (m) {
      fence = { char: m[1][0], len: m[1].length };
      map[i] = true;
    }
  }
  return map;
}

function splitFrontmatter(ls) {
  if (ls[0] !== '---') return null;
  for (let i = 1; i < ls.length; i++) {
    if (ls[i] === '---') return { fm: ls.slice(1, i), bodyStart: i + 1 };
  }
  return null;
}

// Frontmatter as ordered entries. Enough YAML for skill frontmatter: a scalar
// on the key's own line, or a block (list, map, or folded string) carried by
// the indented lines under it. Not a YAML parser, and not trying to be.
function parseFrontmatter(fm) {
  const entries = [];
  for (let i = 0; i < fm.length; i++) {
    const m = fm[i].match(/^([A-Za-z_][\w-]*):[ \t]*(.*)$/);
    if (!m) continue;
    const [, key, rest] = m;
    const block = [];
    for (let j = i + 1; j < fm.length; j++) {
      if (/^[A-Za-z_][\w-]*:/.test(fm[j])) break;
      if (fm[j].trim()) block.push(fm[j]);
    }
    const scalar = rest.trim().replace(/^(["'])(.*)\1$/, '$2');
    entries.push({
      key,
      line: i + 1,
      scalar: scalar || undefined,
      block: block.length ? block : undefined,
      isList: block.length > 0 && block.every((l) => /^\s*- /.test(l)),
      isFolded: /^[|>]/.test(rest.trim()),
    });
  }
  return entries;
}

const fmHas = (fm, key) => parseFrontmatter(fm).some((e) => e.key === key && (e.scalar || e.block));

function fmValue(fm, key) {
  const e = parseFrontmatter(fm).find((x) => x.key === key);
  return e ? e.scalar : undefined;
}

// The frontmatter of a unit file is its host's, so it is checked against the
// host's field table rather than against awa's own vocabulary.
function checkFrontmatter(file, fm, F, errs, warns) {
  const base = basename(file);
  const profile = F.profiles.find((p) => (p.match?.basename || []).includes(base))
    || F.profiles.find((p) => (p.match?.suffix || []).some((s) => base.endsWith(s)))
    || F.profiles.find((p) => p.default);
  if (!profile) return;

  const entries = parseFrontmatter(fm);
  const seen = new Map();
  const bools = new Set(F.booleanValues.map((v) => v.toLowerCase()));

  for (const e of entries) {
    if (seen.has(e.key)) errs.push(`frontmatter: duplicate key "${e.key}" (lines ${seen.get(e.key)} and ${e.line})`);
    seen.set(e.key, e.line);

    const def = F.keys[e.key];
    if (!def) { errs.push(`frontmatter: unknown key "${e.key}" — not a Claude Code skill field`); continue; }
    if (profile.allow && !profile.allow.includes(e.key)) {
      errs.push(`frontmatter: "${e.key}" is not accepted in a ${profile.name} file`);
      continue;
    }
    if (e.scalar === undefined && e.block === undefined) { errs.push(`frontmatter: "${e.key}" has no value`); continue; }

    const isBlock = e.block !== undefined && e.scalar === undefined;
    switch (def.type) {
      case 'boolean':
        if (isBlock || !bools.has((e.scalar || '').toLowerCase())) {
          errs.push(`frontmatter: "${e.key}" must be a boolean (${F.booleanValues.join(', ')}), got "${e.scalar ?? '(block)'}"`);
        }
        break;
      case 'map':
        if (!isBlock || e.isList) errs.push(`frontmatter: "${e.key}" must be a map`);
        break;
      case 'stringOrList':
        if (isBlock && !e.isList) errs.push(`frontmatter: "${e.key}" must be a string or a YAML list`);
        break;
      default: // string
        if (isBlock && !e.isFolded) errs.push(`frontmatter: "${e.key}" must be a string`);
    }

    const value = e.scalar;
    if (value !== undefined) {
      if (def.enum && !def.enum.includes(value)) {
        errs.push(`frontmatter: "${e.key}" is "${value}", not one of ${def.enum.join(', ')}`);
      }
      if (def.pattern && !new RegExp(def.pattern).test(value)) {
        errs.push(`frontmatter: "${e.key}" is "${value}", which does not match ${def.pattern}`);
      }
      if (def.maxLength && value.length > def.maxLength) {
        warns.push(`frontmatter: "${e.key}" is ${value.length} characters, over the ${def.maxLength} the host keeps`);
      }
    }
    if (F.portability && def.portable !== true) {
      warns.push(`frontmatter: "${e.key}" is ${F.portability.warn}`);
    }
  }

  for (const key of profile.required || []) {
    if (!seen.has(key)) errs.push(`frontmatter: ${key} missing (required in a ${profile.name} file)`);
  }

  if (profile.nameMatchesDirectory && seen.has('name')) {
    const dir = basename(dirname(resolve(file)));
    const name = fmValue(fm, 'name');
    if (name && name !== dir) {
      errs.push(`frontmatter: name "${name}" does not match the skill directory "${dir}", which is what the command is named after`);
    }
  }
}

const norm = (t) => t.replace(/\s+/g, ' ').trim();

// Text with fenced blocks blanked; inline code spans kept, so that a tool name
// in backticks is still visible to the load-hoist check.
function unfenced(ls, fenced) {
  return ls.map((l, i) => (fenced[i] ? '' : l)).join('\n');
}

// Text with fenced blocks and inline code spans blanked, for tag scanning:
// `<name>` inside a code span is a CLI placeholder, not an element. A span may
// wrap across a line, so the blanking must not stop at the newline.
function proseOnly(ls, fenced) {
  return unfenced(ls, fenced).replace(/`[^`]*`/g, (s) => s.replace(/[^\n]/g, ' '));
}

// The ```yaml fences of a schema file.
function extractYaml(text) {
  const ls = text.split('\n');
  const fmr = splitFrontmatter(ls);
  if (!fmr) return { yams: [], marks: [] };
  const yams = [];
  const marks = []; // the opening marker of each yaml fence, in the same order
  let inF = false, marker = null, cur = null, curLine = 0;
  for (let i = fmr.bodyStart; i < ls.length; i++) {
    const m = ls[i].match(/^(`{3,}|~{3,})\s*(\S*)\s*$/);
    if (!inF && m) { inF = true; marker = m[1]; cur = m[2] === 'yaml' ? [] : null; curLine = i + 1; continue; }
    if (inF && m && m[1][0] === marker[0] && m[1].length >= marker.length && m[2] === '') {
      inF = false;
      if (cur) { yams.push(cur.join('\n')); marks.push({ marker, line: curLine }); }
      cur = null;
      continue;
    }
    if (inF && cur) cur.push(ls[i]);
  }
  return { yams, marks, fmr };
}

const TAG_RE = /<(\/)?([A-Za-z_][\w-]*)((?:\s+[\w-]+\s*=\s*"[^"]*")*)\s*(\/)?>/g;

function parseAttrs(attrText) {
  const attrs = {};
  for (const m of attrText.matchAll(/([\w-]+)\s*=\s*"([^"]*)"/g)) attrs[m[1]] = m[2];
  return attrs;
}

// Stack-based scan of the prose into element nodes, each with its attributes,
// immediate parent, and (for block elements) its body.
function parseElements(prose, errs) {
  const nodes = [];
  const stack = [];
  const spans = [];
  for (const m of prose.matchAll(TAG_RE)) {
    spans.push([m.index, m.index + m[0].length]);
    const [, close, name, attrText, self] = m;
    if (close) {
      const openIdx = [...stack].reverse().findIndex((n) => n.name === name);
      if (openIdx === -1) { errs.push(`unbalanced </${name}>`); continue; }
      const depth = stack.length - 1 - openIdx;
      if (depth !== stack.length - 1) {
        errs.push(`</${name}> closes across an unclosed <${stack[stack.length - 1].name}>`);
      }
      const node = stack[depth];
      node.body = prose.slice(node.bodyStart, m.index);
      node.closed = true;
      stack.length = depth;
      continue;
    }
    const node = {
      name,
      attrs: parseAttrs(attrText),
      self: Boolean(self),
      body: '',
      parent: stack.length ? stack[stack.length - 1].name : null,
      index: m.index,
      raw: m[0],
    };
    nodes.push(node);
    if (!node.self) { node.bodyStart = m.index + m[0].length; stack.push(node); }
  }
  for (const n of stack) errs.push(`unclosed <${n.name}>`);
  // Anything tag-shaped the strict scanner did not consume is malformed —
  // an unquoted attribute value, a stray angle bracket, a broken close tag.
  for (const m of prose.matchAll(/<\/?[A-Za-z_][\w-]*/g)) {
    if (!spans.some(([a, b]) => m.index >= a && m.index < b)) {
      errs.push(`malformed tag near "${prose.slice(m.index, m.index + 60).split('\n')[0]}"`);
    }
  }
  return nodes;
}

function fmtLabel(tpl, node, text) {
  return tpl.replace(/\{(@?[\w-]+)(?::(\d+))?\}/g, (_, key, cut) => {
    const v = key === 'text' ? text : (key.startsWith('@') ? node.attrs[key.slice(1)] : '') || '';
    return cut ? v.slice(0, +cut) : v;
  });
}

// Resolve a compare "part": "@attr", "body", or "@attr|body" (attr, else body).
function comparePart(part, node) {
  for (const alt of part.split('|')) {
    const v = alt === 'body' ? node.body : node.attrs[alt.slice(1)];
    if (v !== undefined && norm(v)) return norm(v);
  }
  return '';
}

// ---------- unit ----------

function checkUnit(file, text, errs, warns, G) {
  const K = G.kinds.unit;
  const ls = text.split('\n');
  const fenced = fenceMap(ls);
  const fmr = splitFrontmatter(ls);
  if (!fmr) { errs.push('missing YAML frontmatter'); return null; }

  checkFrontmatter(file, fmr.fm, K.frontmatter, errs, warns);

  const prose = proseOnly(ls, fenced);
  const nodes = parseElements(prose, errs);
  const defs = K.elements;

  for (const [tag, message] of Object.entries(K.removed || {})) {
    if (nodes.some((n) => n.name === tag)) errs.push(message);
  }

  const counts = {};
  for (const node of nodes) {
    const def = defs[node.name];
    if (!def) {
      if (!K.removed || !K.removed[node.name]) {
        errs.push(`unknown tag <${node.name}> (unit files use only ${Object.keys(defs).join('/')})`);
      }
      continue;
    }
    counts[node.name] = (counts[node.name] || 0) + 1;

    const want = def.parent === undefined ? [null] : [].concat(def.parent);
    if (!want.includes(node.parent)) {
      const names = want.map((w) => (w ? `<${w}>` : '(the file root)')).join(' or ');
      errs.push(`<${node.name}> must sit directly inside ${names}, found inside <${node.parent || '(the file root)'}>`);
    }

    if (def.form === 'self-closing' && !node.self) {
      errs.push(`<${node.name}> must be self-closing: ${node.raw.slice(0, 70)}`);
    }
    if (def.form === 'block' && node.self) {
      errs.push(`<${node.name}> must have a body: ${node.raw.slice(0, 70)}`);
    }
    if (def.bodyRequired && !norm(node.body)) errs.push(`<${node.name}> is empty`);
    if (def.selfClosingNeedsAttr && node.self && Object.keys(node.attrs).length === 0) {
      errs.push(`self-closing <${node.name} /> needs at least one attribute: ${node.raw}`);
    }
    if (def.bodyForbid && norm(node.body)) {
      const re = new RegExp(def.bodyForbid.pattern, def.bodyForbid.flags || '');
      if (re.test(norm(node.body))) {
        errs.push(`<${node.name}>: ${def.bodyForbid.message}: "${norm(node.body).slice(0, 60)}"`);
      }
    }

    const attrDefs = def.attrs || {};
    for (const [k, v] of Object.entries(node.attrs)) {
      const ad = attrDefs[k];
      if (!ad) {
        const renamed = (G.conditions.renamedFrom || {})[k];
        if (renamed && attrDefs[G.conditions.attribute]) {
          errs.push(`<${node.name}> attribute "${k}" ${renamed}: ${node.raw.slice(0, 70)}`);
        } else {
          errs.push(`<${node.name}> unknown attribute "${k}": ${node.raw.slice(0, 70)}`);
        }
        continue;
      }
      if (ad.const && v !== ad.const) errs.push(`<${node.name}> ${k} must be ${ad.const}, got "${v}"`);
      if (ad.enum && !ad.enum.includes(v)) errs.push(`bad ${node.name} ${k} "${v}": ${node.raw.slice(0, 70)}`);
      if (ad.condition && !new RegExp(G.conditions.pattern).test(v)) {
        errs.push(`bad ${G.conditions.attribute} value "${v}" on <${node.name}> (use ${G.conditions.forms.map((f) => f.split(' —')[0]).join(', ')})`);
      }
      if (/[<>]/.test(v) && (K.checks['attributes-free-of-angle-brackets'] || {}).enabled) {
        errs.push(`${K.checks['attributes-free-of-angle-brackets'].message}: ${k}="${v.slice(0, 60)}"`);
      }
    }
    for (const [k, ad] of Object.entries(attrDefs)) {
      if (ad.required && node.attrs[k] === undefined) {
        errs.push(`<${node.name}> missing ${k} attribute: ${node.raw.slice(0, 70)}`);
      }
    }
    if (def.atMostOneOf) {
      const present = def.atMostOneOf.filter((k) => node.attrs[k] !== undefined);
      if (present.length > 1) {
        errs.push(`<${node.name}> takes at most one of ${def.atMostOneOf.join('/')}: ${node.raw.slice(0, 70)}`);
      }
    }
    if (def.exactlyOneOf) {
      const label = (k) => (k === 'body' ? 'a body' : `\`${k.slice(1)}\``);
      const present = def.exactlyOneOf.filter((k) => (k === 'body' ? norm(node.body) : node.attrs[k.slice(1)] !== undefined));
      const ident = node.attrs.name ? `<${node.name} name="${node.attrs.name}">` : `<${node.name}>`;
      if (present.length !== 1) {
        const what = present.length === 0
          ? `needs exactly one of ${def.exactlyOneOf.map(label).join(', ')}`
          : `has ${present.map(label).join(' and ')}; use one`;
        errs.push(`${ident} ${what}: ${node.raw.slice(0, 60)}`);
      }
    }
    if (def.mustContain) {
      const kids = nodes.filter((n) => n.parent === node.name && n.index > node.index
        && n.index < node.index + node.raw.length + node.body.length);
      const ok = def.mustContain.anyOf.some((t) => kids.some((k) => k.name === t))
        || (def.mustContain.orPattern && new RegExp(def.mustContain.orPattern, 'm').test(node.body));
      if (!ok) {
        const wanted = def.mustContain.anyOf.map((t) => `<${t}>`).join(', ');
        errs.push(`<${node.name}> has no ${wanted}${def.mustContain.orPattern ? ', or bullet' : ''} entries`);
      }
    }
  }

  for (const [tag, def] of Object.entries(defs)) {
    const occ = def.occurs;
    if (!occ) continue;
    const n = counts[tag] || 0;
    if (occ.min !== undefined && n < occ.min) errs.push(`missing <${tag}> block`);
    if (occ.max !== undefined && n > occ.max) errs.push(`expected at most ${occ.max} <${tag}> block, found ${n}`);
  }

  // Frontmatter keys that mirror an element attribute.
  for (const mir of K.frontmatter.mirrors || []) {
    if (mir.basename && basename(file) !== mir.basename) continue;
    const node = nodes.find((n) => n.name === mir.element);
    const fv = fmValue(fmr.fm, mir.key);
    if (node && fv !== undefined && node.attrs[mir.attr] !== undefined && fv !== node.attrs[mir.attr]) {
      errs.push(`frontmatter ${mir.key} "${fv}" does not match <${mir.element} ${mir.attr}="${node.attrs[mir.attr]}">`);
    }
  }

  // Element order.
  const order = K.order || [];
  const seen = order.map((t) => ({ t, at: (nodes.find((n) => n.name === t) || {}).index }))
    .filter((x) => x.at !== undefined);
  for (let i = 1; i < seen.length; i++) {
    if (seen[i].at < seen[i - 1].at) { errs.push(`<${seen[i].t}> must come after <${seen[i - 1].t}>`); break; }
  }

  // Structure is read from the prose, but a statement's wording includes the
  // code spans the prose scan blanks — so the comparables and the load check
  // read the same elements off the unfenced text.
  const rawNodes = parseElements(unfenced(ls, fenced), []);

  // A step whose whole effect is loading context is a mis-routed tool_call.
  const load = K.checks['steps-are-not-pure-loads'];
  if (load && load.enabled) {
    const verbs = load.verbs.map((v) => v.replace(/\s+/g, '\\s+')).join('|');
    const starts = new RegExp(`^(${verbs})\\b`);
    const mentions = new RegExp(`\`?(${G.tools.roster.join('|')})\`?`);
    for (const node of rawNodes) {
      if (node.name !== 'step') continue;
      const body = norm(node.attrs.task !== undefined ? node.attrs.task : node.body);
      if (starts.test(body) && mentions.test(body)) {
        errs.push(`${load.message}: "${body.slice(0, 80)}"`);
      }
    }
  }

  return rawNodes;
}

// ---------- schema ----------

function checkSchema(file, text, errs, G) {
  const K = G.kinds.schema;
  const D = K.document;
  const ls = text.split('\n');
  const fmr = splitFrontmatter(ls);
  if (!fmr) { errs.push('missing AOKF frontmatter'); return; }
  for (const key of K.frontmatter.required || []) {
    if (!fmHas(fmr.fm, key)) errs.push(`frontmatter: ${key} missing`);
  }
  for (const key of K.frontmatter.slug || []) {
    const line = fmr.fm.find((l) => new RegExp(`^${key}:`).test(l));
    if (line && !new RegExp(`^${key}:\\s*[a-z0-9]+(-[a-z0-9]+)*\\s*$`).test(line)) {
      errs.push(`frontmatter: ${key} is not a slug: "${line}"`);
    }
  }

  const { yams, marks } = extractYaml(text);
  if (yams.length !== D.fences) {
    errs.push(`expected exactly ${D.fences} yaml fence, found ${yams.length}`);
    return;
  }
  const want = D['fence-marker'];
  for (const { marker, line } of marks) {
    if (marker !== want) {
      const n = (s) => `${s.length} ${s[0] === '`' ? 'backtick' : 'tilde'}${s.length === 1 ? '' : 's'}`;
      const why =
        marker.length < want.length
          ? ` — the example inside it must be able to carry a fenced block of its own, and a marker this short is closed by the first one`
          : '';
      errs.push(`line ${line}: the yaml contract opens with ${n(marker)}, expected ${n(want)}${why}`);
    }
  }
  let y;
  try { y = YAML.parse(yams[0]); } catch (e) { errs.push(`schema yaml: ${e.message}`); return; }
  if (!y || typeof y !== 'object') { errs.push('schema yaml: not a map'); return; }

  checkKeys(y, D.keys, 'schema yaml', errs, D);
  if (y.preamble === undefined && y.sections === undefined) {
    errs.push('schema yaml: declares neither preamble nor sections — a governed document is one, the other, or both');
  }
  if (y.preamble && typeof y.preamble === 'object' && !Array.isArray(y.preamble)) {
    checkKeys(y.preamble, D.preamble.keys, 'schema yaml: preamble', errs, D);
  }
  if (Array.isArray(y.sections)) {
    if (y.sections.length === 0) errs.push('schema yaml: no sections entries');
    y.sections.forEach((s, i) => {
      const where = `schema yaml: sections[${i}]`;
      if (!s || typeof s !== 'object') { errs.push(`${where}: not a map`); return; }
      checkKeys(s, D.section.keys, where, errs, D);
      const present = D.section.exactlyOneOf.filter((k) => s[k] !== undefined);
      if (present.length !== 1) {
        errs.push(`${where}: needs exactly one of ${D.section.exactlyOneOf.join('/')}, found ${present.length ? present.join(' and ') : 'neither'}`);
      }
    });
    if (!y.sections.some((s) => s && s.required === true)) {
      errs.push('schema yaml: no section marked required: true');
    }
  }
  if (y.frontmatter && typeof y.frontmatter === 'object') {
    for (const [key, c] of Object.entries(y.frontmatter)) {
      if (!c || typeof c !== 'object') { errs.push(`schema yaml: frontmatter.${key}: not a map`); continue; }
      checkKeys(c, D['frontmatter-constraint'].keys, `schema yaml: frontmatter.${key}`, errs, D);
    }
  }
}

// One map against a declared key table: unknown keys, missing required keys,
// declared types, enums, regex validity, and cross-key requirements.
function checkKeys(obj, table, where, errs, D) {
  for (const [key, v] of Object.entries(obj)) {
    const def = table[key];
    if (!def) {
      errs.push(`${where}: unknown key "${key}" (the grammar declares ${Object.keys(table).join(', ')})`);
      continue;
    }
    const got = Array.isArray(v) ? 'list' : v === null ? 'null'
      : Number.isInteger(v) ? 'integer' : typeof v === 'object' ? 'map' : typeof v;
    if (def.type && got !== def.type && !(def.type === 'string' && got === 'integer')) {
      errs.push(`${where}.${key}: expected ${def.type}, got ${got}`);
      continue;
    }
    if (def.enum && !def.enum.includes(v)) {
      errs.push(`${where}.${key}: ${JSON.stringify(v)} is not one of ${def.enum.join(', ')}`);
    }
    if (def.format === 'regex') {
      try { new RegExp(v); } catch (e) { errs.push(`${where}.${key}: not a valid regex — ${e.message}`); }
    }
    for (const [rk, rv] of Object.entries(def.requires || {})) {
      if (obj[rk] !== rv) errs.push(`${where}.${key}: only allowed with ${rk}: ${rv}`);
    }
  }
  for (const [key, def] of Object.entries(table)) {
    if (def.required && obj[key] === undefined) errs.push(`${where}: missing required key "${key}"`);
  }
}

// ---------- core ----------

const CORE_BLOCKS = new Set();

function checkCore(file, text, errs, G) {
  const K = G.kinds.core;
  if (K.collectBlocks) {
    for (const m of text.matchAll(/<([a-z][a-z0-9_]*)(?:\s[^>]*)?>/g)) CORE_BLOCKS.add(m[1]);
  }
  const ls = text.split('\n');
  const fenced = fenceMap(ls);
  if (K.requireH1 && !ls.some((l, i) => !fenced[i] && /^# /.test(l))) errs.push('core: missing H1');
  if (!K.balancedTags) return;
  const prose = proseOnly(ls, fenced);
  const stack = [];
  for (const m of prose.matchAll(TAG_RE)) {
    const [, close, name, , self] = m;
    if (self) continue;
    if (close) {
      if (stack.length === 0 || stack[stack.length - 1] !== name) {
        errs.push(`core: unbalanced </${name}>`);
        return;
      }
      stack.pop();
    } else {
      stack.push(name);
    }
  }
  if (stack.length) errs.push(`core: unclosed <${stack[stack.length - 1]}>`);
}

// ---------- duplication ----------


function tokset(s, stopWords) {
  const out = new Set();
  for (let t of s.toLowerCase().replace(/[^a-z0-9]+/g, ' ').split(' ')) {
    if (!t || stopWords.has(t)) continue;
    if (t.length > 3 && t.endsWith('s')) t = t.slice(0, -1); // light stemming
    out.add(t);
  }
  return out;
}

function containment(A, B) {
  let i = 0;
  for (const t of A) if (B.has(t)) i++;
  return i / Math.min(A.size, B.size);
}

function unitComparables(nodes, G) {
  const defs = G.kinds.unit.elements;
  const constants = new Set(G.duplication.skeletonConstants);
  const items = [];
  for (const node of nodes) {
    const cmp = (defs[node.name] || {}).compare;
    if (!cmp) continue;
    if (cmp.skipIf) {
      const v = comparePart(cmp.skipIf.part, node);
      if (new RegExp(cmp.skipIf.pattern).test(v)) continue;
    }
    const parts = cmp.parts.map((p) => comparePart(p, node)).filter(Boolean);
    const text = parts.join(' ');
    if (!text || constants.has(text)) continue;
    items.push({ element: node.name, where: fmtLabel(cmp.label, node, text), text });
  }
  return items;
}

function coreComparables(text) {
  const ls = text.split('\n');
  const fenced = fenceMap(ls);
  const items = [];
  ls.forEach((l, i) => {
    if (fenced[i] || /^\s*#/.test(l)) return;
    const stripped = l.replace(/<[^>]*>/g, '').trim();
    if (stripped) items.push({ element: 'line', where: `line ${i + 1}`, text: stripped });
  });
  return items;
}

function schemaComparables(text, G) {
  const { yams } = extractYaml(text);
  if (yams.length !== 1) return [];
  const C = G.kinds.schema.compare;
  let ylines = yams[0].split('\n');
  const stopAt = ylines.findIndex((l) => new RegExp(`^${C.stopAtKey}:`).test(l));
  if (stopAt >= 0) ylines = ylines.slice(0, stopAt); // the example is illustration, not a statement
  const items = [];
  for (let i = 0; i < ylines.length; i++) {
    const m = ylines[i].match(new RegExp(`^(\\s*)${C.descriptionKey}:\\s*([>|])?\\s*(.*)$`));
    if (!m) continue;
    const where = `yaml ${C.descriptionKey} (line ${i + 1})`;
    if (m[3] && !m[2]) { items.push({ element: 'description', where, text: m[3] }); continue; }
    const indent = m[1].length;
    const buf = [];
    for (let j = i + 1; j < ylines.length; j++) {
      if (!ylines[j].trim()) break;
      if (ylines[j].match(/^\s*/)[0].length <= indent) break;
      buf.push(ylines[j].trim());
    }
    if (buf.length) items.push({ element: 'description', where, text: buf.join(' ') });
  }
  return items;
}

function checkDuplication(comparables, G) {
  const D = G.duplication;
  const within = new Set(D.withinFileKinds);
  const cross = new Set(D.crossPairs.flatMap((p) => {
    const [a, b] = p.split('|');
    return [`${a}|${b}`, `${b}|${a}`];
  }));
  const exempt = new Set(D.exemptCrossUnitElements);
  const findings = [];
  for (let i = 0; i < comparables.length; i++) {
    for (let j = i + 1; j < comparables.length; j++) {
      const A = comparables[i], B = comparables[j];
      if (A.file === B.file) {
        if (!within.has(A.kind)) continue;
      } else if (!cross.has(`${A.kind}|${B.kind}`)) {
        continue;
      } else if (A.kind === 'unit' && B.kind === 'unit' && (exempt.has(A.element) || exempt.has(B.element))) {
        continue; // D.exemptCrossUnitReason
      }
      if (Math.min(A.tokens.size, B.tokens.size) < D.minTokens) continue;
      const sim = containment(A.tokens, B.tokens);
      if (sim >= D.threshold) findings.push({ A, B, sim });
    }
  }
  return findings;
}

// ---------- the grammar itself ----------
//
// A typo in the grammar would otherwise switch a rule off silently, so the
// grammar is checked against superdev-grammar.meta.yaml before it is used. The
// checker below covers the JSON Schema subset that meta-schema uses — enough
// to be dependency-free, not a general implementation.

function resolveRef(ref, root) {
  if (!ref.startsWith('#/')) throw new Error(`unsupported $ref ${ref}`);
  let node = root;
  for (const part of ref.slice(2).split('/')) node = node[part.replace(/~1/g, '/').replace(/~0/g, '~')];
  if (!node) throw new Error(`unresolved $ref ${ref}`);
  return node;
}

function typeOf(v) {
  if (v === null) return 'null';
  if (Array.isArray(v)) return 'array';
  if (Number.isInteger(v)) return 'integer';
  return typeof v;
}

function schemaErrors(value, schema, root, path, errs) {
  if (schema.$ref) schemaErrors(value, resolveRef(schema.$ref, root), root, path, errs);
  for (const sub of schema.allOf || []) schemaErrors(value, sub, root, path, errs);

  if (schema.type) {
    const want = Array.isArray(schema.type) ? schema.type : [schema.type];
    const got = typeOf(value);
    const ok = want.includes(got) || (want.includes('number') && got === 'integer');
    if (!ok) { errs.push(`${path}: expected ${want.join(' or ')}, got ${got}`); return; }
  }
  if (schema.const !== undefined && value !== schema.const) {
    errs.push(`${path}: must be ${JSON.stringify(schema.const)}`);
  }
  if (schema.enum && !schema.enum.includes(value)) {
    errs.push(`${path}: ${JSON.stringify(value)} is not one of ${schema.enum.map((e) => JSON.stringify(e)).join(', ')}`);
  }
  if (typeof value === 'string') {
    if (schema.minLength !== undefined && value.length < schema.minLength) errs.push(`${path}: must not be empty`);
    if (schema.pattern && !new RegExp(schema.pattern).test(value)) {
      errs.push(`${path}: ${JSON.stringify(value)} does not match ${schema.pattern}`);
    }
    if (schema.format === 'regex') {
      try { new RegExp(value); } catch (e) { errs.push(`${path}: not a valid regex — ${e.message}`); }
    }
  }
  if (typeof value === 'number' && schema.minimum !== undefined && value < schema.minimum) {
    errs.push(`${path}: must be >= ${schema.minimum}`);
  }
  if (Array.isArray(value)) {
    if (schema.minItems !== undefined && value.length < schema.minItems) {
      errs.push(`${path}: needs at least ${schema.minItems} item(s)`);
    }
    if (schema.items) value.forEach((v, i) => schemaErrors(v, schema.items, root, `${path}[${i}]`, errs));
  }
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const keys = Object.keys(value);
    if (schema.minProperties !== undefined && keys.length < schema.minProperties) {
      errs.push(`${path}: needs at least ${schema.minProperties} propert(ies)`);
    }
    for (const key of schema.required || []) {
      if (!(key in value)) errs.push(`${path}: missing required key "${key}"`);
    }
    const pats = Object.entries(schema.patternProperties || {});
    for (const key of keys) {
      const sub = (schema.properties || {})[key];
      const byPat = pats.filter(([p]) => new RegExp(p).test(key));
      if (sub) schemaErrors(value[key], sub, root, `${path}.${key}`, errs);
      for (const [, s] of byPat) schemaErrors(value[key], s, root, `${path}.${key}`, errs);
      if (!sub && byPat.length === 0 && schema.additionalProperties === false && (schema.properties || schema.patternProperties)) {
        errs.push(`${path}: unknown key "${key}"`);
      }
    }
  }
}

// Cross-references a JSON Schema cannot state: a name used in one part of the
// grammar must be declared in another.
function grammarSemanticErrors(G) {
  const errs = [];
  const U = G.kinds.unit;
  const els = U.elements;
  const has = (t) => Object.prototype.hasOwnProperty.call(els, t);
  const attrsOf = (t) => Object.keys((els[t] || {}).attrs || {});
  const ref = (t) => `kinds.unit.elements.${t}`;

  if (!has(U.root)) errs.push(`kinds.unit.root: <${U.root}> is not a declared element`);
  for (const t of U.order) if (!has(t)) errs.push(`kinds.unit.order: <${t}> is not a declared element`);

  for (const [tag, def] of Object.entries(els)) {
    for (const p of def.parent === undefined ? [] : [].concat(def.parent)) {
      if (p !== null && !has(p)) errs.push(`${ref(tag)}.parent: <${p}> is not a declared element`);
    }
    if (def.parent === null && tag !== U.root) {
      errs.push(`${ref(tag)}.parent: only the root <${U.root}> may sit at the file root`);
    }
    for (const t of (def.mustContain || {}).anyOf || []) {
      if (!has(t)) errs.push(`${ref(tag)}.mustContain: <${t}> is not a declared element`);
    }
    for (const a of def.atMostOneOf || []) {
      if (!attrsOf(tag).includes(a)) errs.push(`${ref(tag)}.atMostOneOf: "${a}" is not an attribute of <${tag}>`);
    }
    const partAttrs = (s) => s.split('|').filter((p) => p.startsWith('@')).map((p) => p.slice(1));
    const named = [
      ...(def.exactlyOneOf || []).flatMap(partAttrs).map((a) => ['exactlyOneOf', a]),
      ...((def.compare || {}).parts || []).flatMap(partAttrs).map((a) => ['compare.parts', a]),
      ...partAttrs(((def.compare || {}).skipIf || {}).part || '').map((a) => ['compare.skipIf.part', a]),
    ];
    for (const [where, a] of named) {
      if (!attrsOf(tag).includes(a)) errs.push(`${ref(tag)}.${where}: "@${a}" is not an attribute of <${tag}>`);
    }
    for (const [a, ad] of Object.entries(def.attrs || {})) {
      if (ad.condition && a !== G.conditions.attribute) {
        errs.push(`${ref(tag)}.attrs.${a}: only "${G.conditions.attribute}" may carry a condition`);
      }
      if (a === G.conditions.attribute && !ad.condition) {
        errs.push(`${ref(tag)}.attrs.${a}: the condition attribute must be marked condition: true`);
      }
      if ((G.conditions.renamedFrom || {})[a]) {
        errs.push(`${ref(tag)}.attrs.${a}: "${a}" is listed in conditions.renamedFrom, so it cannot also be live`);
      }
    }
  }

  for (const t of G.duplication.exemptCrossUnitElements) {
    if (!has(t)) errs.push(`duplication.exemptCrossUnitElements: <${t}> is not a declared element`);
  }
  const F = U.frontmatter;
  const fmKey = (k) => Object.prototype.hasOwnProperty.call(F.keys, k);
  for (const p of F.profiles) {
    for (const k of p.required || []) {
      if (!fmKey(k)) errs.push(`kinds.unit.frontmatter.profiles.${p.name}.required: "${k}" is not a declared frontmatter key`);
    }
    for (const k of p.allow || []) {
      if (!fmKey(k)) errs.push(`kinds.unit.frontmatter.profiles.${p.name}.allow: "${k}" is not a declared frontmatter key`);
    }
    for (const k of p.required || []) {
      if (p.allow && !p.allow.includes(k)) {
        errs.push(`kinds.unit.frontmatter.profiles.${p.name}: "${k}" is required but not in allow`);
      }
    }
    if (p.nameMatchesDirectory && !fmKey('name')) {
      errs.push(`kinds.unit.frontmatter.profiles.${p.name}: nameMatchesDirectory needs a declared "name" key`);
    }
  }
  const fmDefaults = F.profiles.filter((p) => p.default);
  if (fmDefaults.length !== 1) {
    errs.push(`kinds.unit.frontmatter.profiles: exactly one profile must carry default: true, found ${fmDefaults.length}`);
  }
  for (const m of F.mirrors || []) {
    if (!fmKey(m.key)) errs.push(`kinds.unit.frontmatter.mirrors: "${m.key}" is not a declared frontmatter key`);
    if (!has(m.element)) errs.push(`kinds.unit.frontmatter.mirrors: <${m.element}> is not a declared element`);
    else if (!attrsOf(m.element).includes(m.attr)) {
      errs.push(`kinds.unit.frontmatter.mirrors: "${m.attr}" is not an attribute of <${m.element}>`);
    }
  }
  // A document key declared `of: X` must name a sibling key table.
  const D = G.kinds.schema.document;
  for (const [key, def] of Object.entries(D.keys)) {
    if (def.of && !(def.of in D)) {
      errs.push(`kinds.schema.document.keys.${key}.of: "${def.of}" is not a declared key table`);
    }
  }
  for (const table of ['section', 'frontmatter-constraint']) {
    for (const k of D[table].exactlyOneOf || []) {
      if (!(k in D[table].keys)) {
        errs.push(`kinds.schema.document.${table}.exactlyOneOf: "${k}" is not a declared key`);
      }
    }
    for (const [k, def] of Object.entries(D[table].keys)) {
      for (const rk of Object.keys(def.requires || {})) {
        if (!(rk in D[table].keys)) {
          errs.push(`kinds.schema.document.${table}.keys.${k}.requires: "${rk}" is not a declared key`);
        }
      }
    }
  }

  const defaults = Object.entries(G.kinds).filter(([, K]) => K.match.default);
  if (defaults.length !== 1) {
    errs.push(`kinds: exactly one kind must carry match.default, found ${defaults.length}`);
  }
  return errs;
}

// ---------- grammar reference ----------

function renderDoc(G) {
  const out = [];
  out.push(`# ${G.grammar} grammar ${G.version}`, '', G.doc, '');
  out.push('## Conditions', '', G.conditions.doc, '');
  out.push(`One attribute, \`${G.conditions.attribute}\`, on every element that can bear a condition:`, '');
  for (const f of G.conditions.forms) out.push(`- \`${f.split(' —')[0]}\` —${f.split(' —')[1] || ''}`);
  out.push('');
  for (const [from, note] of Object.entries(G.conditions.renamedFrom || {})) {
    out.push(`\`${from}\` is not part of the grammar: ${note}.`);
  }
  const F = G.kinds.unit.frontmatter;
  out.push('', '## Unit frontmatter', '', F.doc, '');
  for (const p of F.profiles) {
    const where = (p.match?.basename || []).concat(p.match?.suffix || []).map((s) => `\`${s}\``).join(', ');
    out.push(`- **${p.name}** — ${p.doc}${where ? ` Matches ${where}.` : ''} Requires ${p.required.map((k) => `\`${k}\``).join(', ') || 'nothing'}.`);
  }
  out.push('');
  for (const [key, def] of Object.entries(F.keys)) {
    const bits = [def.type];
    if (def.enum) bits.push(`one of ${def.enum.join(', ')}`);
    if (def.pattern) bits.push(`matching \`${def.pattern}\``);
    if (def.maxLength) bits.push(`at most ${def.maxLength} chars`);
    if (!def.portable) bits.push(`${F.portability.spec}: no`);
    out.push(`- \`${key}\` — ${bits.join('; ')}${def.doc ? `. ${def.doc}` : ''}`);
  }
  out.push('', `Keys marked "${F.portability.spec}: no" are ${F.portability.warn} (${F.portability.url}).`, '');

  out.push('', '## Unit elements', '');
  for (const [tag, def] of Object.entries(G.kinds.unit.elements)) {
    const attrs = Object.entries(def.attrs || {})
      .map(([k, a]) => `${k}="…"${a.required ? '' : '?'}`).join(' ');
    const form = def.form === 'self-closing' ? `<${tag} ${attrs} />`
      : def.form === 'block' ? `<${tag}${attrs ? ` ${attrs}` : ''}>…</${tag}>`
        : `<${tag} ${attrs} />  |  <${tag} …>…</${tag}>`;
    out.push(`### \`${tag}\``, '', '```xml', form, '```', '', def.doc);
    if (def.parent) out.push(`Sits inside ${[].concat(def.parent).map((p) => `\`<${p}>\``).join(' or ')}.`);
    if (def.occurs) out.push(`Occurs ${def.occurs.min}–${def.occurs.max === undefined ? 'n' : def.occurs.max} times.`);
    for (const [k, a] of Object.entries(def.attrs || {})) {
      if (a.doc || a.enum || a.const) {
        const extra = a.enum ? ` One of ${a.enum.join(', ')}.` : a.const ? ` Always \`${a.const}\`.` : '';
        out.push(`- \`${k}\`${a.required ? ' (required)' : ''} — ${a.doc || ''}${extra}`);
      }
    }
    out.push('');
  }
  out.push('## Element order', '', G.kinds.unit.order.map((t) => `\`${t}\``).join(' → '), '');
  return out.join('\n');
}

// ---------- main ----------

// The kind that claims `file`: a string, `null` when a kind claims the name but
// excepts it, or `undefined` when nothing claims it at all. A bare run walks the
// grammar's roots and checks only what is claimed, so an unrelated markdown file
// beside a claimed one is passed over rather than read as the fallback kind.
function detectKind(file, G, claimed = true) {
  const b = basename(file);
  const parent = basename(dirname(resolve(file)));
  let fallback = 'unit';
  for (const [kind, K] of Object.entries(G.kinds)) {
    const claims = (K.match.basename || []).includes(b)
      || (K.match.suffix || []).some((s) => b.endsWith(s))
      || (K.match.dir || []).includes(parent);
    if (claims) return (K.match.except || []).includes(b) ? null : kind;
    if (K.match.default) fallback = kind;
  }
  return claimed ? fallback : undefined;
}

const argv = process.argv.slice(2);
let kindOverride = null;
let grammarPath = DEFAULT_GRAMMAR;
let metaPath = null;
let wantDoc = false;
let wantJson = false;
const files = [];
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === '--kind') kindOverride = argv[++i];
  else if (argv[i] === '--grammar') grammarPath = argv[++i];
  else if (argv[i] === '--meta') metaPath = argv[++i];
  else if (argv[i] === '--doc') wantDoc = true;
  else if (argv[i] === '--json') wantJson = true;
  else files.push(argv[i]);
}
if (!metaPath) metaPath = grammarPath.replace(/\.(json|ya?ml)$/, '') + '.meta.yaml';

const parseData = (text, path) => (/\.ya?ml$/.test(path) ? YAML.parse(text) : JSON.parse(text));

let G, META;
try {
  G = parseData(readFileSync(grammarPath, 'utf8'), grammarPath);
} catch (e) {
  console.error(`cannot read grammar ${grammarPath}: ${e.message}`);
  process.exit(2);
}
try {
  META = parseData(readFileSync(metaPath, 'utf8'), metaPath);
} catch (e) {
  console.error(`cannot read meta-schema ${metaPath}: ${e.message}`);
  process.exit(2);
}

const gErrs = [];
try {
  schemaErrors(G, META, META, 'grammar', gErrs);
  if (gErrs.length === 0) gErrs.push(...grammarSemanticErrors(G));
} catch (e) {
  gErrs.push(`meta-schema could not be applied: ${e.message}`);
}
if (gErrs.length) {
  console.error(`INVALID GRAMMAR  ${grammarPath}  (against ${basename(metaPath)})`);
  for (const e of gErrs) console.error(`  ERROR: ${e}`);
  process.exit(2);
}

if (wantDoc) { console.log(renderDoc(G)); process.exit(0); }

if (files.length === 0 && G.roots) {
  const walk = (dir) => {
    let out = [];
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, e.name);
      if (e.isDirectory()) out = out.concat(walk(full));
      else if (detectKind(full, G, false) !== undefined) out.push(full);
    }
    return out;
  };
  for (const root of G.roots.paths) {
    if (existsSync(root)) files.push(...walk(root).sort());
  }
}

if (files.length === 0) {
  console.error('usage: validate-awa.mjs [--grammar <file>] [--kind unit|schema|core] <file...>');
  console.error('       validate-awa.mjs --doc');
  process.exit(2);
}

const stopWords = new Set(G.duplication.stopWords);
// Every finding, for --json. The shape matches one entry of the AOKF report's
// `findings`, so a golden captured here pins the texts and the JSON together.
const jsonFindings = [];
let failed = false;
const comparables = [];
const texts = [];

for (const file of files) {
  const errs = [], warns = [];
  const kind = kindOverride || detectKind(file, G);
  if (kind === null) { console.log(`SKIP  [-]       ${file}`); continue; }
  if (!G.kinds[kind]) { console.error(`unknown kind: ${kind}`); process.exit(2); }
  if (!existsSync(file)) {
    errs.push('file not found');
  } else {
    const text = readFileSync(file, 'utf8');
    texts.push({ file, kind, text });
    let items = [];
    if (kind === 'unit') {
      const nodes = checkUnit(file, text, errs, warns, G);
      if (nodes) items = unitComparables(nodes, G);
    } else if (kind === 'schema') {
      checkSchema(file, text, errs, G);
      items = schemaComparables(text, G);
    } else if (kind === 'core') {
      checkCore(file, text, errs, G);
      items = coreComparables(text);
    }
    for (const c of items) comparables.push({ file, kind, ...c, tokens: tokset(c.text, stopWords) });
  }
  for (const e of errs) jsonFindings.push({ severity: 'error', file, message: e });
  for (const w of warns) jsonFindings.push({ severity: 'warning', file, message: w });
  if (!wantJson) {
    const status = errs.length ? 'FAIL' : 'PASS';
    console.log(`${status}  [${kind}]  ${file}`);
    for (const e of errs) console.log(`  ERROR: ${e}`);
    for (const w of warns) console.log(`  warn:  ${w}`);
  }
  if (errs.length) failed = true;
}

// A "core <x> block" reference must resolve to a block the core defines.
const refCheck = G.kinds.unit.checks['core-block-references'];
if (CORE_BLOCKS.size && refCheck && refCheck.enabled) {
  const over = new Set(refCheck.appliesTo || ['unit']);
  for (const { file, kind, text } of texts) {
    if (!over.has(kind)) continue;
    for (const m of text.matchAll(new RegExp(refCheck.pattern, 'g'))) {
      if (!CORE_BLOCKS.has(m[1])) {
        const message = `${refCheck.message}: <${m[1]}>`;
        jsonFindings.push({ severity: 'error', file, message });
        if (!wantJson) {
          console.log(`FAIL  [${kind}]  ${file}`);
          console.log(`  ERROR: ${message}`);
        }
        failed = true;
      }
    }
  }
}

const dups = checkDuplication(comparables, G);
if (dups.length) {
  failed = true;
  if (!wantJson) console.log('DUPLICATION');
  for (const { A, B, sim } of dups) {
    const pct = (sim * 100).toFixed(0);
    const message =
      `${pct}% overlap — one occurrence must become a reference: ` +
      `${A.file} (${A.where}): "${A.text.slice(0, 90)}" | ` +
      `${B.file} (${B.where}): "${B.text.slice(0, 90)}"`;
    jsonFindings.push({ severity: 'error', file: A.file, message });
    if (!wantJson) {
      console.log(`  ERROR: ${pct}% overlap — one occurrence must become a reference:`);
      console.log(`    ${A.file} (${A.where}): "${A.text.slice(0, 90)}"`);
      console.log(`    ${B.file} (${B.where}): "${B.text.slice(0, 90)}"`);
    }
  }
}

if (wantJson) {
  console.log(
    JSON.stringify({ files: files.length, passed: !failed, findings: jsonFindings }, null, 2),
  );
}
process.exit(failed ? 1 : 0);
