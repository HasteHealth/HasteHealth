/*
 * Closes a navbar dropdown when one of its items is clicked.
 *
 * Docusaurus dropdowns open on hover. Following a link swaps the page without
 * a reload, so the pointer is still over the menu and it stays open on the
 * new page. After a click, keep the menu hidden until the pointer leaves.
 */

const DISMISSED = "data-dismissed";

function onClick(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const link = target.closest(".navbar .dropdown__link");
  const dropdown = link?.closest<HTMLElement>(".dropdown");
  if (!link || !dropdown) return;

  // An attribute, not a class: React owns className and rewrites it.
  dropdown.setAttribute(DISMISSED, "");
  dropdown.addEventListener("mouseleave", () => dropdown.removeAttribute(DISMISSED), {
    once: true,
  });

  // A menu opened from the keyboard stays open until a mousedown outside it.
  document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  (link as HTMLElement).blur();
}

if (typeof document !== "undefined") {
  document.addEventListener("click", onClick);
}
