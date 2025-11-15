import React from "react";

import { Button, useHasteHealth } from "@haste-health/components";

export default function EmptyWorkspace() {
  const hasteHealth = useHasteHealth();

  return (
    <div className="h-screen w-screen flex  flex-col items-center">
      <div className=" flex justify-center items-center flex-col px-4 py-4 shadow-md -top-[15px] mt-16">
        <h1 className="text-xl font-semibold mb-4 ">
          There are no tenants associated with your account.
        </h1>
        <Button
          buttonType="secondary"
          onClick={() => hasteHealth.logout(window.location.origin)}
        >
          Logout
        </Button>
      </div>
    </div>
  );
}
