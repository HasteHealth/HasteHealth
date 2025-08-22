import React from "react";

import { Navigation } from "./Navigation";

export interface ShellProps {
  children: React.ReactNode;
}

export const Shell = ({ children }: ShellProps) => {
  return (
    <>
      <div className="min-h-full">
        <Navigation
          navigation={[{ name: "Dashboard" }, { name: "Resources" }]}
          userNavigation={[{ name: "Settings" }, { name: "Sign out" }]}
          active="Dashboard"
          user={{
            email: "jane@doe.com",
            name: "Jane Doe",
          }}
        />

        <header>
          <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
            <h1 className="text-3xl font-bold tracking-tight text-gray-900">
              Dashboard
            </h1>
          </div>
        </header>
        <main>
          <div className="bg-gray-100 mx-auto max-w-7xl py-6 sm:px-6 lg:px-8 h-full">
            {children}
          </div>
        </main>
      </div>
    </>
  );
};
