import React from "react";

export const Logo = ({
  className = "text-brand-500",
  onClick,
}: Readonly<{ className?: string; onClick?: () => void }>) => (
  <svg
    viewBox="0 0 430 160"
    preserveAspectRatio="xMidYMid meet"
    role="img"
    xmlns="http://www.w3.org/2000/svg"
    className={className}
    onClick={onClick}
  >
    <title>Haste Health logo</title>
    <desc>
      Haste Health logo with three teal pulse bars beside the wordmark
      haste.health
    </desc>

    <g transform="translate(17, 80)">
      <rect
        x="0"
        y="-28"
        width="24"
        height="56"
        rx="6"
        fill="#0d9488"
        opacity="0.45"
      />
      <rect x="32" y="-52" width="24" height="104" rx="6" fill="#0d9488" />
      <rect
        x="64"
        y="-18"
        width="24"
        height="36"
        rx="6"
        fill="#0d9488"
        opacity="0.7"
      />

      <text
        x="108"
        y="19"
        fontFamily="system-ui, -apple-system, sans-serif"
        fontSize="54"
        fontWeight="700"
        fill="#0f172a"
      >
        haste
        <tspan fontWeight="300" fill="#94a3b8">
          .health
        </tspan>
      </text>
    </g>
  </svg>
);
