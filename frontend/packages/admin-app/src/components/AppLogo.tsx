import React from "react";

import { useTenantBranding } from "../hooks/useTenantBranding";
import { Logo } from "./Logo";

export const AppLogo = ({
  className,
  onClick,
}: Readonly<{ className?: string; onClick?: () => void }>) => {
  const { logoDataUrl, loading } = useTenantBranding();

  if (loading) {
    return <div className={className} aria-hidden="true" />;
  }

  if (logoDataUrl) {
    return (
      <img
        src={logoDataUrl}
        alt="Tenant logo"
        className={className}
        onClick={onClick}
      />
    );
  }

  return <Logo className={className} onClick={onClick} />;
};
