import React from "react";

interface CardProps {
  title: string;
  usage: number;
  limit: number;
  description: string;
}

export default function Card({
  title,
  usage,
  limit,
  description,
}: Readonly<CardProps>) {
  return (
    <div className="text-slate-800 w-full px-4 md:w-1/2 xl:w-1/4">
      <div className="h-[300px] relative mb-10 overflow-hidden rounded-[10px] border-solid border-2 border-stroke  py-10 px-8 shadow-pricing sm:p-12 lg:py-10 lg:px-6 xl:p-[50px]">
        <span className="mb-6 block text-lg font-semibold text-primary truncate">
          {title}
        </span>
        <h2 className="mb-10 text-3xl font-bold text-dark ">
          <span>{new Intl.NumberFormat().format(usage)}</span>
          <span className="text-base font-medium text-body-color ">
            {" "}
            / {new Intl.NumberFormat().format(limit)}
          </span>
        </h2>
        <p className="mb-2 text-sm text-body-color ">{description}</p>
      </div>
    </div>
  );
}
