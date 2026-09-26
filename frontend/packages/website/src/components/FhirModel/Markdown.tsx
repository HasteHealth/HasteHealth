import React from "react";

/*
 * FHIR definitions are short markdown: paragraphs, `*` lists, links, bold and
 * code. Their links are relative to the HL7 R4 spec ("patient.html"). This
 * renders that subset without pulling a markdown parser into every page.
 */

const HL7_BASE = "https://hl7.org/fhir/R4/";
const INLINE = /\[([^\]]+)\]\(([^)\s]+)\)|\*\*([^*]+)\*\*|`([^`]+)`/g;

function resolve(href: string): string {
  return /^[a-z]+:/i.test(href) || href.startsWith("#") ? href : HL7_BASE + href;
}

function inline(text: string): React.ReactNode[] {
  const nodes: React.ReactNode[] = [];
  let last = 0;
  for (const match of text.matchAll(INLINE)) {
    const [whole, label, href, bold, code] = match;
    const index = match.index ?? 0;
    if (index > last) nodes.push(text.slice(last, index));
    if (href) {
      nodes.push(
        <a key={index} href={resolve(href)} target="_blank" rel="noopener noreferrer">
          {label}
        </a>,
      );
    } else if (bold) {
      nodes.push(<strong key={index}>{bold}</strong>);
    } else {
      nodes.push(<code key={index}>{code}</code>);
    }
    last = index + whole.length;
  }
  if (last < text.length) nodes.push(text.slice(last));
  return nodes;
}

type Block = { list: boolean; lines: string[] };

function blocks(text: string): Block[] {
  const result: Block[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) {
      result.push({ list: false, lines: [] });
      continue;
    }
    const list = /^[*-]\s+/.test(line);
    const current = result.at(-1);
    if (current?.list === list && current.lines.length > 0) {
      current.lines.push(line);
    } else {
      result.push({ list, lines: [line] });
    }
  }
  return result.filter((block) => block.lines.length > 0);
}

export default function Markdown({
  text,
  className,
}: Readonly<{ text: string; className?: string }>) {
  return (
    <div className={className}>
      {blocks(text).map((block, i) =>
        block.list ? (
          <ul key={i}>
            {block.lines.map((line, j) => (
              <li key={j}>{inline(line.replace(/^[*-]\s+/, ""))}</li>
            ))}
          </ul>
        ) : (
          <p key={i}>{inline(block.lines.join(" "))}</p>
        ),
      )}
    </div>
  );
}
