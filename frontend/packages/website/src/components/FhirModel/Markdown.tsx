import React from "react";

/*
 * FHIR definitions are short markdown: paragraphs, `*` lists, links, bold and
 * code. Their links are relative to the HL7 R4 spec ("patient.html"). This
 * renders that subset without pulling a markdown parser into every page.
 */

const HL7_BASE = "https://hl7.org/fhir/R4/";

function resolve(href: string): string {
  return /^[a-z]+:/i.test(href) || href.startsWith("#") ? href : HL7_BASE + href;
}

type Token = { at: number; end: number; node: React.ReactNode };

/** `[label](href)`, `**bold**` or `` `code` `` starting at `at`, if any. */
function tokenAt(text: string, at: number): Token | undefined {
  if (text[at] === "[") {
    const close = text.indexOf("](", at + 1);
    const end = close === -1 ? -1 : text.indexOf(")", close + 2);
    const href = end === -1 ? "" : text.slice(close + 2, end);
    if (close > at + 1 && href && !/\s/.test(href)) {
      return {
        at,
        end: end + 1,
        node: (
          <a key={at} href={resolve(href)} target="_blank" rel="noopener noreferrer">
            {text.slice(at + 1, close)}
          </a>
        ),
      };
    }
  } else if (text.startsWith("**", at)) {
    const end = text.indexOf("**", at + 2);
    if (end > at + 2) {
      return { at, end: end + 2, node: <strong key={at}>{text.slice(at + 2, end)}</strong> };
    }
  } else if (text[at] === "`") {
    const end = text.indexOf("`", at + 1);
    if (end > at + 1) {
      return { at, end: end + 1, node: <code key={at}>{text.slice(at + 1, end)}</code> };
    }
  }
  return undefined;
}

/** Scans once, left to right; no regex backtracking. Keys are offsets. */
function inline(text: string): React.ReactNode[] {
  const nodes: React.ReactNode[] = [];
  let last = 0;
  let at = 0;
  while (at < text.length) {
    const token = tokenAt(text, at);
    if (token) {
      if (at > last) nodes.push(text.slice(last, at));
      nodes.push(token.node);
      last = token.end;
      at = token.end;
    } else {
      at += 1;
    }
  }
  if (last < text.length) nodes.push(text.slice(last));
  return nodes;
}

/** A paragraph or list, with the source line it starts on (its key). */
type Block = { list: boolean; line: number; lines: { line: number; text: string }[] };

const LIST_ITEM = /^[*-]\s+/;

function blocks(text: string): Block[] {
  const result: Block[] = [];
  text.split(/\r?\n/).forEach((raw, line) => {
    const trimmed = raw.trim();
    if (!trimmed) {
      result.push({ list: false, line, lines: [] });
      return;
    }
    const list = LIST_ITEM.test(trimmed);
    const current = result.at(-1);
    if (current?.list === list && current.lines.length > 0) {
      current.lines.push({ line, text: trimmed });
    } else {
      result.push({ list, line, lines: [{ line, text: trimmed }] });
    }
  });
  return result.filter((block) => block.lines.length > 0);
}

export default function Markdown({
  text,
  className,
}: Readonly<{ text: string; className?: string }>) {
  return (
    <div className={className}>
      {blocks(text).map((block) =>
        block.list ? (
          <ul key={block.line}>
            {block.lines.map((item) => (
              <li key={item.line}>{inline(item.text.replace(LIST_ITEM, ""))}</li>
            ))}
          </ul>
        ) : (
          <p key={block.line}>{inline(block.lines.map((item) => item.text).join(" "))}</p>
        ),
      )}
    </div>
  );
}
