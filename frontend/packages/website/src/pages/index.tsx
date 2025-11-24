import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header
      style={{
        backgroundColor: "transparent",
      }}
    >
      <Heading
        as="h1"
        className="mt-12 hero__title text-orange-950 text-center"
      >
        {siteConfig.title}
      </Heading>
      <div className="mb-8 text-center">
        <span className="text-lg text-orange-950 font-semibold">
          Modern healthcare development platform. Built for{" "}
          <span className="text-orange-600 ">performance</span> and{" "}
          <span className="text-orange-600 ">scale</span>.
        </span>
      </div>
      {/* <p className="hero__subtitle text--secondary">{siteConfig.tagline}</p> */}
      <div className="flex justify-center items-center space-x-4">
        <Link
          className="button button--secondary button--lg  border"
          to="/docs/intro"
        >
          Getting Started - 5min ⏱️
        </Link>
      </div>
    </header>
  );
}

function DescriptionColumn(props: { title: string; description: string }) {
  return (
    <div className="space-y-1">
      <div className="text-2xl font-semibold underline">{props.title}</div>
      <span className="text-sm">{props.description}</span>
    </div>
  );
}

function CarouselCard(props: { onClick?: () => void; children?: ReactNode }) {
  return (
    <div
      className="carousel-card cursor-pointer flex items-center justify-center "
      onClick={props.onClick}
    >
      {props.children}
    </div>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();

  return (
    <Layout
      wrapperClassName="bg-orange-50"
      title={`Haste Health`}
      description="Description will go into a meta tag in <head />"
    >
      <div className="container mx-auto px-2 ">
        <HomepageHeader />
        <main className="mt-12 z-1 text-orange-950">
          <div id="tw-scope" className="mt-4">
            <div className="space-y-20">
              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    Easily{" "}
                    <span className="text-orange-600 underline">
                      interoperate
                    </span>{" "}
                    with other healthcare systems
                  </h3>
                  <div className="grid md:grid-cols-2 grid-cols-1 gap-4 mt-4 py-4">
                    <DescriptionColumn
                      title="FHIR"
                      description="Built from the ground up to support the FHIR (Fast Healthcare Interoperability Resources) a modern, open standard created by HL7 to help healthcare systems securely exchange data."
                    />
                    <DescriptionColumn
                      title="Hl7v2"
                      description="Full interoperability with HL7v2 messaging to integrate with legacy healthcare systems."
                    />
                  </div>
                </div>
                <div className="p-6 flex justify-center items-center  border border-slate-200 rounded-lg min-h-72">
                  <div className="carousel basic">
                    <div className="group font-bold text-3xl">
                      <CarouselCard>
                        <span className="text-rose-600 hover:underline ">
                          Epic
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-sky-600  hover:underline ">
                          Cerner
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-slate-700 hover:underline ">
                          Athenahealth
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-emerald-600 hover:underline ">
                          Meditech
                        </span>
                      </CarouselCard>
                    </div>
                    <div className="group  font-bold text-3xl">
                      <CarouselCard>
                        <span className="text-rose-600 hover:underline ">
                          Epic
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-sky-600 hover:underline ">
                          Cerner
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-slate-700 hover:underline ">
                          Athenahealth
                        </span>
                      </CarouselCard>
                      <CarouselCard>
                        <span className="text-emerald-600 hover:underline ">
                          Meditech
                        </span>
                      </CarouselCard>
                    </div>
                  </div>
                </div>
              </div>

              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="p-6 justify-center border border-slate-200 rounded-lg min-h-72 grid grid-cols-2 gap-2">
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-4xl font-bold">
                      {"<10"}
                      <span className="text-sm">ms</span>
                    </h3>
                    <span>
                      Average latency for updating/creating resources.
                    </span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-4xl font-bold">
                      {"20k"} <span className="text-sm">resources/second</span>
                    </h3>
                    <span>
                      Throughput per instance in our load tests running on 10
                      threads.
                    </span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-4xl font-bold">
                      {"<50"}
                      <span className="text-sm">ms</span>
                    </h3>
                    <span>For most parameter/value search requests.</span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-4xl font-bold">
                      {"<100"}
                      <span className="text-sm">mb</span>
                    </h3>
                    <span>Memory usage for a single instance.</span>
                  </div>
                </div>
                <div className="space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    High performance with{" "}
                    <span className="text-green-600">low latency</span> that can
                    scale to millions of patients.
                  </h3>
                </div>
              </div>

              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    Support for authentication with{" "}
                    <span className="text-blue-600 underline">OIDC</span> and{" "}
                    <span className="text-blue-600 underline">
                      SMART on FHIR
                    </span>
                  </h3>
                  <div className="grid md:grid-cols-3 grid-cols-1 gap-4 mt-4 py-4">
                    <DescriptionColumn
                      title="Grants"
                      description="Support for Authorization Code, Client Credentials, and Refresh Token grants."
                    />
                    <DescriptionColumn
                      title="Federated login"
                      description="Login with any identity provider that supports OIDC."
                    />
                    <DescriptionColumn
                      title="Scopes"
                      description="Request only the FHIR resource access you need with fine-grained scopes."
                    />
                  </div>
                </div>
                <div className="p-6 rounded-lg ">
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img
                        src="/img/okta.svg"
                        className="h-32 object-contain"
                      />
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img
                        src="/img/azure.svg"
                        className="h-32 object-contain"
                      />
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img
                        src="/img/auth0.svg"
                        className="h-32 object-contain"
                      />
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img
                        src="/img/aws_logo.svg"
                        className="h-32 object-contain"
                      />
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img
                        src="/img/keycloak.png"
                        className="h-32 object-contain"
                      />
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-sm border border-slate-200">
                      <img src="/img/gcp.png" className="h-32 object-contain" />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </main>
      </div>
    </Layout>
  );
}
