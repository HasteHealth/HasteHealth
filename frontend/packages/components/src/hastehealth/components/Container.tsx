import classNames from "classnames";
import React from "react";

export interface ContainerProps {
  title?: string;
  logo?: string;
  children: React.ReactNode;
  size?: "md" | "lg";
}

export const Container = ({
  logo,
  title,
  children,
  size = "md",
}: ContainerProps) => {
  return (
    <section className="bg-gray-50 dark:bg-gray-900 h-screen">
      <div className="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0">
        <a
          href="#"
          className="flex items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white"
        >
          {logo && <img className="w-8 h-8 mr-2" src={logo} alt="logo" />}
          {title}
        </a>
        <div
          className={classNames(
            "w-full bg-white rounded-lg shadow dark:border md:mt-0  xl:p-0 dark:bg-gray-800 dark:border-gray-700",
            { "sm:max-w-md": size === "md", "sm:max-w-2xl": size === "lg" },
          )}
        >
          <div className="p-6 space-y-4 md:space-y-6 sm:p-8">{children}</div>
        </div>
      </div>
    </section>
  );
};
