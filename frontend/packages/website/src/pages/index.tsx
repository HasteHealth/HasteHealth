import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";
import HealthcareDataFlow from "../components/HealthcareDataFlow";

function DescriptionColumn(
  props: Readonly<{
    title: ReactNode;
    description: ReactNode;
  }>,
) {
  return (
    <div className="space-y-1">
      <div className="text-2xl font-bold underline ">{props.title}</div>
      <span className="text-sm">{props.description}</span>
    </div>
  );
}

function CarouselCard(
  props: Readonly<{ onClick?: () => void; children?: ReactNode }>,
) {
  return (
    <div
      className="carousel-card cursor-pointer flex items-center justify-center "
      onClick={props.onClick}
    >
      {props.children}
    </div>
  );
}

function BorderBlock() {
  return (
    <div
      style={{ width: "calc(100vw - 1.1rem)" }}
      className="border-b  border-orange-200 w-screen absolute left-0 -mt-6"
    />
  );
}

function BorderVertical({ height }: Readonly<{ height?: number }>) {
  console.log("height:", height);
  return (
    <div
      style={{ height: height }}
      className="border-0 md:border-l  border-orange-200  absolute left-1/2 -mt-6"
    />
  );
}

export default function Home(): ReactNode {
  const containerRef = React.useRef<HTMLDivElement>(null);
  const [_, setContainerHeight] = React.useState<number | undefined>(undefined);
  React.useEffect(() => {
    if (containerRef.current) {
      let style = window.getComputedStyle(containerRef.current);
      console.log("Container height:", containerRef.current.clientHeight);
      console.log("Container margin top:", style.marginTop);
      setContainerHeight(
        containerRef.current.clientHeight +
          Number.parseInt(style.marginTop, 10) / 2,
      );
    }
  }, [containerRef]);
  const { siteConfig } = useDocusaurusContext();

  return (
    <Layout
      wrapperClassName="bg-orange-50"
      title={`Haste Health`}
      description="Modern healthcare clinical data repository. Built for performance and scale."
    >
      <meta name="algolia-site-verification" content="A94F28B6A640A6FE" />
      <div
        id="tw-scope"
        className="container mx-auto px-4 border-x border-y-0  border-orange-200"
      >
        <main ref={containerRef} className=" z-1 text-orange-950">
          {/* <BorderVertical height={containerHeight} /> */}
          <div className="space-y-16">
            <header className="space-y-6 pt-16 p-8">
              <Heading
                as="h1"
                className="text-6xl md:text-7xl font-bold text-orange-950 "
              >
                {siteConfig.title}
              </Heading>
              <div className="">
                <span className="text-2xl text-orange-950 font-semibold">
                  Is an open source clinical data repository. Built for{" "}
                  <span className="text-orange-600 ">performance</span> and{" "}
                  <span className="text-orange-600 ">scale</span>.
                </span>
              </div>
              <div className="flex flex-col md:flex-row md:items-center space-y-2 md:space-y-0 md:space-x-2">
                <Link
                  className="block text-xl font-semibold text-white border-orange-950  rounded-md px-8 py-2 bg-orange-600 hover:bg-orange-500"
                  to="/docs/getting_started/quick_start"
                >
                  Getting Started - 5min ⏱️
                </Link>

                <Link
                  className="block text-xl font-semibold text-white border-orange-950  rounded-md px-8 py-2 bg-gray-600 hover:bg-gray-500"
                  to="/docs/overview/what_is_haste_health"
                >
                  Documentation 📚
                </Link>
              </div>
            </header>

            <BorderBlock />
            <div className="grid md:grid-cols-2 grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className="space-y-8">
                <div>
                  <h3 className="text-5xl font-bold">
                    Easily store and retrieve healthcare data
                  </h3>
                </div>
                <div>
                  Haste Health acts as a central clinical data repository. It
                  ingests healthcare data from many sources and exposes it
                  through standard APIs that modern healthcare systems
                  understand.
                </div>
              </div>
              <div className="space-y-8">
                <div className="grid md:grid-cols-2 grid-cols-1 gap-4">
                  <DescriptionColumn
                    title={
                      <Link to="/docs/category/fhir">
                        <span className="hover:underline hover:text-orange-500">
                          FHIR
                        </span>
                      </Link>
                    }
                    description="Built from the ground up to support the FHIR (Fast Healthcare Interoperability Resources) a modern, open standard created by HL7 to help healthcare systems securely exchange data."
                  />
                  <DescriptionColumn
                    title="Hl7v2"
                    description="Full interoperability with HL7v2 messaging to integrate with legacy healthcare systems."
                  />
                </div>
              </div>
            </div>
            <BorderBlock />
            <div className="grid grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className="space-y-8">
                <div>
                  <h3 className="text-5xl font-bold">How it works</h3>
                </div>
                <div className="grid md:grid-cols-3 grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                  <div className="border rounded-md border-orange-200 p-6 space-y-2">
                    <div>
                      <h4 className="text-3xl font-semibold">1. Ingest</h4>
                    </div>
                    <div>
                      Connect healthcare systems using FHIR APIs, HL7v2
                      messages, and bulk data exports.
                    </div>
                  </div>
                  <div className="border rounded-md border-orange-200 p-6 space-y-2">
                    <div>
                      <h4 className="text-3xl font-semibold">2. Store</h4>
                    </div>
                    <div>
                      Data is stored in a unified clinical data model optimized
                      for performance and scale.
                    </div>
                  </div>
                  <div className="border rounded-md border-orange-200 p-6 space-y-2">
                    <div>
                      <h4 className="text-3xl font-semibold">3. Access</h4>
                    </div>
                    <div>
                      Your apps access data using FHIR APIs, OAuth & OIDC, and
                      other modern api standards.
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <BorderBlock />

            <div className="grid md:grid-cols-2  grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className="order-1 md:order-2 space-y-8">
                <div>
                  <h3 className="text-5xl font-bold">
                    <span class="text-orange-600 underline">
                      Interoperability
                    </span>{" "}
                    with the healthcare systems you already use
                  </h3>
                </div>
              </div>
              <div className="order-2 md:order-1 justify-center rounded-lg min-h-72 grid grid-cols-2 gap-4">
                <div className="flex flex-col space-y-1">
                  <h3 className="text-xl font-bold underline">
                    Labs and Diagnostics
                  </h3>
                  <span>
                    <span>
                      Exchange results, orders, and clinical observations with
                      lab systems using{" "}
                      <Link
                        to="/docs/integration/healthcare_systems/lab_diagnostics"
                        className="underline text-orange-900 hover:text-orange-800"
                      >
                        standardized FHIR and HL7v2 interfaces
                      </Link>
                      .
                    </span>
                  </span>
                </div>
                <div className="flex flex-col space-y-1">
                  <h3 className="text-xl font-bold underline">
                    Health Information Exchanges
                  </h3>
                  <span>
                    <span>
                      Connect to regional and national health information
                      exchanges to share patient data across healthcare
                      organizations using{" "}
                      <Link
                        to="/docs/integration/healthcare_systems/health_information_exchange"
                        className="underline text-orange-900 hover:text-orange-800"
                      >
                        FHIR-based exchange protocols
                      </Link>
                      .
                    </span>
                  </span>
                </div>
                <div className="flex flex-col space-y-1">
                  <h3 className="text-xl font-bold underline">EHR Systems</h3>
                  <span>
                    <span>
                      Integrate with major EHR systems like Epic, Cerner, and
                      Meditech using{" "}
                      <Link
                        to="/docs/integration/healthcare_systems/ehr"
                        className="underline text-orange-900 hover:text-orange-800"
                      >
                        FHIR APIs and HL7v2 messaging
                      </Link>
                      .
                    </span>
                  </span>
                </div>
                <div className="flex flex-col space-y-1">
                  <h3 className="text-xl font-bold underline">
                    Payer and Insurance Systems
                  </h3>
                  <span>
                    <span>
                      Exchange eligibility, claims, and prior authorization data
                      with payer systems using{" "}
                      <Link
                        to="/docs/integration/healthcare_systems/payers_insurance"
                        className="underline text-orange-900 hover:text-orange-800"
                      >
                        FHIR
                      </Link>
                      .
                    </span>
                  </span>
                </div>
              </div>
            </div>
            <BorderBlock />
            <div className="grid md:grid-cols-1 grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className="space-y-8">
                <h3 className="text-5xl font-bold">
                  Built for real-world healthcare workloads
                </h3>

                <div className="justify-center rounded-lg grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-3xl font-bold">
                      {"<10"}
                      <span className="text-sm">ms</span>
                    </h3>
                    <span>
                      Response time for updating and creating resources.
                    </span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-3xl font-bold">
                      {"20k"} <span className="text-sm">resources/second</span>
                    </h3>
                    <span>
                      Total throughput running on 10 threads using the{" "}
                      <a
                        className="underline text-orange-900 hover:text-orange-800"
                        href="https://synthea.mitre.org/downloads"
                      >
                        synthia data set
                      </a>
                      .
                    </span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-3xl font-bold">
                      {"<50"}
                      <span className="text-sm">ms</span>
                    </h3>
                    <span>For most parameter/value search requests.</span>
                  </div>
                  <div className="flex flex-col space-y-1">
                    <h3 className="text-3xl font-bold">
                      {"<100"}
                      <span className="text-sm">mb</span>
                    </h3>
                    <span>Memory usage for a single instance.</span>
                  </div>
                </div>
              </div>
            </div>
            <BorderBlock />
            <div className="grid md:grid-cols-1  grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className=" rounded-lg space-y-8">
                <div>
                  <h3 className="text-5xl font-bold">
                    <span className="text-orange-600 underline">Secure</span> by
                    design
                  </h3>
                </div>
                <div>
                  <span>
                    Haste Health is built from the ground up with security best
                    practices to help you protect sensitive healthcare data and
                    maintain compliance with healthcare regulations like HIPAA
                    and HITECH.
                  </span>
                </div>
              </div>
              <div className="grid md:grid-cols-2 grid-cols-1 gap-4">
                <DescriptionColumn
                  title={"Compliance"}
                  description="Haste Health is designed to help you meet HIPAA and HITECH requirements for protecting electronic protected health information (ePHI)."
                />
                <DescriptionColumn
                  title={"Best Practices"}
                  description="Built-in support for encryption, access controls, and audit logging to help you secure healthcare data."
                />

                <DescriptionColumn
                  title={
                    <Link
                      className="hover:text-orange-500"
                      to="/docs/category/oauth-grant-types"
                    >
                      OIDC & SMART on FHIR
                    </Link>
                  }
                  description="Industry-standard authentication using OpenID Connect and SMART on FHIR for secure healthcare application integration."
                />
                <DescriptionColumn
                  title={
                    <Link
                      className="hover:text-orange-500"
                      to="/docs/api/authorization/access_control"
                    >
                      Access Policies
                    </Link>
                  }
                  description="Fine-grained authorization controls using AccessPolicy resources to define who can access what healthcare data."
                />
              </div>
            </div>
            <BorderBlock />

            <div className="grid md:grid-cols-2  grid-cols-1 gap-8 grid-flow-row-dense auto-cols-max p-8">
              <div className="order-1 md:order-2 rounded-lg space-y-8">
                <div>
                  <h3 className="text-5xl font-bold">
                    Integration with{" "}
                    <span class="text-orange-600 underline">AI</span>{" "}
                    applications
                  </h3>
                </div>
                <div>
                  <span>
                    Haste Health enables secure real-time clinical context for
                    AI applications.
                  </span>
                </div>
              </div>
              <div className="order-2 md:order-1">
                <div className="grid md:grid-cols-2 grid-cols-1 gap-4">
                  <DescriptionColumn
                    title={"Model Context Protocl"}
                    description="Easily provide LLMs with secure, real-time access to patient data using Haste's Model Context Protocol (MCP) implementation."
                  />
                  <DescriptionColumn
                    title={"Control Data Access"}
                    description="Support for detailed scopes to control exactly what data AI applications can access."
                  />
                </div>
              </div>
            </div>
            <BorderBlock />
            <div className="mt-36" />
          </div>
        </main>
      </div>
    </Layout>
  );
}
