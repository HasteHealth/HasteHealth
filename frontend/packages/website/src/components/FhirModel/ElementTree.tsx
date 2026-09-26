import React, { useMemo, useState } from "react";
import Link from "@docusaurus/Link";

import type { Element, ElementType, ModelData } from "./types";
import styles from "./styles.module.css";

type Row = Element & { name: string; depth: number; parent: boolean };

const anchor = (path: string) => `el-${path}`;

function toRows(elements: Element[]): Row[] {
  return elements.map((element, i) => ({
    ...element,
    name: element.path.slice(element.path.lastIndexOf(".") + 1),
    depth: element.path.split(".").length - 2,
    parent: elements[i + 1]?.path.startsWith(`${element.path}.`) ?? false,
  }));
}

function TypeLink({ name, href }: Readonly<{ name: string; href?: string }>) {
  return href ? <Link to={href}>{name}</Link> : <span>{name}</span>;
}

function TypeName({ type }: Readonly<{ type: ElementType }>) {
  if (!type.targets) return <TypeLink name={type.code} href={type.href} />;
  return (
    <span>
      <TypeLink name={type.code} href={type.href} />(
      {type.targets.map((target, i) => (
        <React.Fragment key={target.name}>
          {i > 0 && " | "}
          <TypeLink name={target.name} href={target.href} />
        </React.Fragment>
      ))}
      )
    </span>
  );
}

function Types({ row }: Readonly<{ row: Row }>) {
  if (row.contentReference) {
    return (
      <span className={styles.muted}>
        Same as <a href={`#${anchor(row.contentReference)}`}>{row.contentReference}</a>
      </span>
    );
  }
  return (
    <span className={styles.types}>
      {row.types.map((type, i) => (
        <React.Fragment key={type.code}>
          {i > 0 && <span className={styles.muted}> | </span>}
          <TypeName type={type} />
        </React.Fragment>
      ))}
    </span>
  );
}

/** HL7 value sets link to their R4 page; others show their canonical URL. */
function valueSetHref(url: string): string | undefined {
  const prefix = "http://hl7.org/fhir/ValueSet/";
  if (!url.startsWith(prefix)) return undefined;
  return `https://hl7.org/fhir/R4/valueset-${url.slice(prefix.length)}.html`;
}

function Binding({ binding }: Readonly<{ binding: NonNullable<Element["binding"]> }>) {
  const name = binding.valueSet.split("/").pop();
  const href = valueSetHref(binding.valueSet);
  return (
    <span className={styles.binding}>
      Binding ({binding.strength}):{" "}
      {href ? (
        <a href={href} target="_blank" rel="noopener noreferrer">
          {name}
        </a>
      ) : (
        <span title={binding.valueSet}>{name}</span>
      )}
    </span>
  );
}

function Flags({ row }: Readonly<{ row: Row }>) {
  return (
    <span className={styles.flags}>
      {row.min > 0 && (
        <span className={styles.required} title="Required">
          Required
        </span>
      )}
      {row.modifier && (
        <span className={styles.flag} title="Modifier: can change the meaning of the resource">
          ?!
        </span>
      )}
      {row.summary && (
        <span className={styles.flag} title="Included in _summary results">
          Σ
        </span>
      )}
    </span>
  );
}

function Chevron({ open }: Readonly<{ open: boolean }>) {
  return (
    <svg
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
      className={open ? `${styles.chevron} ${styles.chevronOpen}` : styles.chevron}
    >
      <path
        fillRule="evenodd"
        d="M7.2 14.8a.75.75 0 0 1 0-1.06L10.94 10 7.2 6.26a.75.75 0 1 1 1.06-1.06l4.27 4.27a.75.75 0 0 1 0 1.06L8.26 14.8a.75.75 0 0 1-1.06 0Z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function ElementRow({
  row,
  open,
  onToggle,
}: Readonly<{ row: Row; open: boolean; onToggle: (path: string) => void }>) {
  return (
    <div
      id={anchor(row.path)}
      className={styles.row}
      role="row"
      style={{ "--depth": row.depth } as React.CSSProperties}
    >
      <div className={styles.nameCell} role="cell">
        {row.parent ? (
          <button
            type="button"
            className={styles.toggle}
            aria-expanded={open}
            aria-label={`${open ? "Collapse" : "Expand"} ${row.name}`}
            onClick={() => onToggle(row.path)}
          >
            <Chevron open={open} />
          </button>
        ) : (
          <span className={styles.toggleSpacer} />
        )}
        <span className={row.inherited ? `${styles.name} ${styles.inherited}` : styles.name}>
          {row.name}
        </span>
        <Flags row={row} />
      </div>
      <div className={styles.cardCell} role="cell">
        <span className={row.min > 0 ? styles.cardRequired : undefined}>
          {row.min}..{row.max}
        </span>
      </div>
      <div className={styles.typeCell} role="cell">
        <Types row={row} />
      </div>
      <div className={styles.descriptionCell} role="cell">
        {row.short}
        {row.binding && <Binding binding={row.binding} />}
      </div>
    </div>
  );
}

export default function ElementTree({ data }: Readonly<{ data: ModelData }>) {
  const rows = useMemo(() => toRows(data.elements), [data]);
  const inheritedCount = rows.filter((row) => row.inherited).length;
  const parents = rows.filter((row) => row.parent).map((row) => row.path);

  // Resources hide the Resource/DomainResource plumbing by default; on data
  // types those few elements are the point of the page.
  const [showInherited, setShowInherited] = useState(data.kind !== "resource");
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());

  const toggle = (path: string) =>
    setCollapsed((current) => {
      const next = new Set(current);
      if (!next.delete(path)) next.add(path);
      return next;
    });

  const visible = rows.filter(
    (row) =>
      (showInherited || !row.inherited) &&
      ![...collapsed].some((path) => row.path.startsWith(`${path}.`)),
  );

  return (
    <div className={styles.root}>
      <div className={styles.toolbar}>
        {inheritedCount > 0 && (
          <label className={styles.checkbox}>
            <input
              type="checkbox"
              checked={showInherited}
              onChange={(e) => setShowInherited(e.target.checked)}
            />
            <span>Show inherited elements ({inheritedCount})</span>
          </label>
        )}
        {parents.length > 0 && (
          <span className={styles.toolbarButtons}>
            <button type="button" className={styles.button} onClick={() => setCollapsed(new Set())}>
              Expand all
            </button>
            <button type="button" className={styles.button} onClick={() => setCollapsed(new Set(parents))}>
              Collapse all
            </button>
          </span>
        )}
      </div>
      <div className={styles.tree} role="table" aria-label={`${data.name} elements`}>
        <div className={`${styles.row} ${styles.headRow}`} role="row">
          <div role="columnheader">Element</div>
          <div role="columnheader">Card.</div>
          <div role="columnheader">Type</div>
          <div role="columnheader">Description</div>
        </div>
        {visible.map((row) => (
          <ElementRow key={row.path} row={row} open={!collapsed.has(row.path)} onToggle={toggle} />
        ))}
      </div>
      <p className={styles.legend}>
        <span className={styles.flag}>Σ</span> in <code>_summary</code> results ·{" "}
        <span className={styles.flag}>?!</span> modifier element · <strong>1..</strong> required ·
        inherited elements in <span className={styles.inherited}>grey</span>
      </p>
    </div>
  );
}
