import {
  type CSSProperties,
  type RefObject,
  useEffect,
  useRef,
  useState,
} from "react";

const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

function motionAllowed(): boolean {
  return (
    "IntersectionObserver" in window &&
    !window.matchMedia(REDUCED_MOTION).matches
  );
}

/**
 * Scroll reveal for everything under the returned ref that carries a
 * `data-reveal` attribute. The styles are in src/css/custom.css.
 *
 * The markup is server-rendered fully visible, and it stays that way unless
 * this hook runs: an element is only hidden ("armed") if it is still below the
 * fold when the page hydrates, and it is released the first time it scrolls
 * into view. Anything already on screen is left alone, so nothing blinks out
 * and back in on load, and a reader with reduced motion, or with no
 * JavaScript at all, just gets the page.
 */
export function useScrollReveal<T extends HTMLElement>(): RefObject<T | null> {
  const ref = useRef<T>(null);

  useEffect(() => {
    const root = ref.current;
    if (!root || !motionAllowed()) return;

    const fold = window.innerHeight;
    const targets = Array.from(
      root.querySelectorAll<HTMLElement>("[data-reveal]"),
    ).filter((element) => element.getBoundingClientRect().top > fold);

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          // An attribute, not a class: React owns className and rewrites it.
          (entry.target as HTMLElement).dataset.reveal = "in";
          observer.unobserve(entry.target);
        }
      },
      { rootMargin: "0px 0px -4% 0px", threshold: 0.05 },
    );

    for (const element of targets) {
      element.dataset.reveal = "armed";
      observer.observe(element);
    }

    return () => {
      observer.disconnect();
      for (const element of targets) element.dataset.reveal = "";
    };
  }, []);

  return ref;
}

/** Staggers a revealed element behind its siblings: `style={revealDelay(i)}`. */
export function revealDelay(index: number, step = 70): CSSProperties {
  return { "--reveal-delay": `${index * step}ms` } as CSSProperties;
}

/**
 * When a piece of the hero rises into place, in milliseconds after page load:
 * `style={riseDelay(ms)}` on an element with the `rise` class
 * (src/components/site/styles.module.css).
 */
export function riseDelay(ms: number): CSSProperties {
  return { "--rise-delay": `${ms}ms` } as CSSProperties;
}

const COUNT_MS = 1100;

/**
 * A number that counts up from zero the first time it scrolls into view.
 *
 * Unless it is mid-count it renders the real value as plain text: on the
 * server, with reduced motion or no JavaScript, when it is already on screen
 * at load, and again once the count has finished. While it counts, screen
 * readers are given the final value, never a figure caught part-way.
 */
export function CountUp({
  value,
  format = (n: number) => n.toLocaleString("en-US"),
}: Readonly<{ value: number; format?: (n: number) => string }>) {
  const ref = useRef<HTMLSpanElement>(null);
  // The figure on screen while counting; null when showing the real value.
  const [counting, setCounting] = useState<number | null>(null);

  useEffect(() => {
    const element = ref.current;
    if (!element || !motionAllowed()) return;
    if (element.getBoundingClientRect().top <= window.innerHeight) return;

    let frame = 0;
    setCounting(0);

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        observer.disconnect();
        const started = performance.now();
        const tick = (now: number) => {
          const progress = Math.min(1, (now - started) / COUNT_MS);
          if (progress === 1) {
            setCounting(null);
            return;
          }
          // Ease out: fast at first, settling onto the final figure.
          setCounting(Math.round(value * (1 - (1 - progress) ** 4)));
          frame = requestAnimationFrame(tick);
        };
        frame = requestAnimationFrame(tick);
      },
      { threshold: 0.5 },
    );
    observer.observe(element);

    const finish = () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
      setCounting(null);
    };
    // A printed page is never scrolled, so it must not be left at zero.
    window.addEventListener("beforeprint", finish);

    return () => {
      window.removeEventListener("beforeprint", finish);
      finish();
    };
  }, [value]);

  if (counting === null) {
    return <span ref={ref}>{format(value)}</span>;
  }

  return (
    <span ref={ref}>
      <span className="sr-only">{format(value)}</span>
      <span aria-hidden="true">{format(counting)}</span>
    </span>
  );
}
