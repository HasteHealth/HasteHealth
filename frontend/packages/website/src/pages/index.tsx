import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

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
          to="/docs/Getting Started/Intro"
        >
          Getting Started - 5min ⏱️
        </Link>
      </div>
    </header>
  );
}

function DescriptionColumn(
  props: Readonly<{
    title: ReactNode;
    description: ReactNode;
  }>
) {
  return (
    <div className="space-y-1">
      <div className="text-2xl font-semibold underline ">{props.title}</div>
      <span className="text-sm">{props.description}</span>
    </div>
  );
}

function CarouselCard(
  props: Readonly<{ onClick?: () => void; children?: ReactNode }>
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
      className="border-b border-dashed border-orange-200 w-screen absolute left-0 -mt-6"
    />
  );
}

function BorderVertical({ height }: { height?: number }) {
  console.log("height:", height);
  return (
    <div
      style={{ height: height }}
      className="border-0 md:border-l border-dashed border-orange-200  absolute left-1/2 -mt-6"
    />
  );
}

export default function Home(): ReactNode {
  const containerRef = React.useRef<HTMLDivElement>(null);
  const [containerHeight, setContainerHeight] = React.useState<
    number | undefined
  >(undefined);
  React.useEffect(() => {
    if (containerRef.current) {
      let style = window.getComputedStyle(containerRef.current);
      console.log("Container height:", containerRef.current.clientHeight);
      console.log("Container margin top:", style.marginTop);
      setContainerHeight(
        containerRef.current.clientHeight + parseInt(style.marginTop, 10) / 2
      );
    }
  }, [containerRef]);

  return (
    <Layout
      wrapperClassName="bg-orange-50"
      title={`Haste Health`}
      description="Description will go into a meta tag in <head />"
    >
      <meta name="algolia-site-verification" content="A94F28B6A640A6FE" />
      <div className="container mx-auto px-4 border-x border-y-0 border-dashed border-orange-200">
        <HomepageHeader />

        <main ref={containerRef} className="mt-12 z-1 text-orange-950">
          <div id="tw-scope" className="mt-4">
            <BorderVertical height={containerHeight} />
            <div className="space-y-16">
              <BorderBlock />
              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    Easily{" "}
                    <Link to="/docs/category/ehr">
                      <span className="text-orange-600 underline hover:text-orange-500">
                        interoperate
                      </span>
                    </Link>{" "}
                    with other healthcare systems
                  </h3>
                  <div className="grid md:grid-cols-2 grid-cols-1 gap-4 mt-4 py-4">
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
                <div className="p-6 flex justify-center items-center  rounded-lg min-h-72">
                  <div className="carousel basic">
                    <div className="group font-bold text-3xl">
                      <Link to="/docs/integration/EHR/Epic">
                        <CarouselCard>
                          <span className="text-rose-700 hover:underline ">
                            Epic Systems
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Cerner">
                        <CarouselCard>
                          <span className="text-sky-600  hover:underline ">
                            Cerner
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Athenahealth">
                        <CarouselCard>
                          <span className="text-slate-700 hover:underline ">
                            Athenahealth
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Meditech">
                        <CarouselCard>
                          <span className="text-emerald-600 hover:underline ">
                            Meditech
                          </span>
                        </CarouselCard>
                      </Link>
                    </div>
                    <div className="group  font-bold text-3xl">
                      <Link to="/docs/integration/EHR/Epic">
                        <CarouselCard>
                          <span className="text-rose-700 hover:underline ">
                            Epic Systems
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Cerner">
                        <CarouselCard>
                          <span className="text-sky-600  hover:underline ">
                            Cerner
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Athenahealth">
                        <CarouselCard>
                          <span className="text-slate-700 hover:underline ">
                            Athenahealth
                          </span>
                        </CarouselCard>
                      </Link>
                      <Link to="/docs/integration/EHR/Meditech">
                        <CarouselCard>
                          <span className="text-emerald-600 hover:underline ">
                            Meditech
                          </span>
                        </CarouselCard>
                      </Link>
                    </div>
                  </div>
                </div>
              </div>

              <BorderBlock />
              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="order-2 md:order-1 p-6 justify-center rounded-lg min-h-72 grid grid-cols-2 gap-4">
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
                <div className="order-1 md:order-2 space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    High performance with{" "}
                    <span className="text-green-600">low latency</span> that can
                    scale to millions of patients.
                  </h3>
                </div>
              </div>
              <BorderBlock />
              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    Built in support for connecting to{" "}
                    <Link to="/docs/category/ai-integrations">
                      <span className="text-purple-600 hover:text-purple-500 underline">
                        AI Applications
                      </span>{" "}
                    </Link>
                  </h3>
                  <div className="grid md:grid-cols-2 grid-cols-1 gap-4 mt-4 py-4">
                    <DescriptionColumn
                      title={
                        <Link
                          className="hover:text-purple-500"
                          to="/docs/API/Model%20Context%20Protocol/Endpoint"
                        >
                          Model Context Protocol
                        </Link>
                      }
                      description="Easily provide LLMs with secure, real-time access to patient data using Haste's Model Context Protocol (MCP) implementation."
                    />
                    <DescriptionColumn
                      title={
                        <Link
                          className="hover:text-purple-500"
                          to="/docs/Authentication/Scopes"
                        >
                          Control data access
                        </Link>
                      }
                      description="Support for detailed scopes to control exactly what data AI applications can access."
                    />
                  </div>
                </div>
                <div className="p-6 rounded-lg ">
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/openai-integration">
                        <img
                          src="/img/openai_logo.svg"
                          alt="OpenAI Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/claude-integration">
                        <img
                          src="/img/claude_logo.svg"
                          alt="Claude Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/gemini-integration">
                        <img
                          src="/img/gemini_logo.svg"
                          alt="Gemini Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/mistral-integration">
                        <img
                          src="/img/mistral_logo.svg"
                          alt="Mistral Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/github-copilot-integration">
                        <img
                          src="/img/copilot_logo.svg"
                          alt="CoPilot Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/AI%20Applications/deepseek-integration">
                        <img
                          src="/img/deepseek_logo.svg"
                          alt="DeepSeek Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                  </div>
                </div>
              </div>
              <BorderBlock />
              <div className="grid md:grid-cols-2  grid-cols-1 gap-4 grid-flow-row-dense auto-cols-max">
                <div className="order-2 md:order-1 p-6 rounded-lg ">
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/Okta">
                        <img
                          src="/img/okta.svg"
                          alt="Okta Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/azure-integration">
                        <img
                          src="/img/azure.svg"
                          alt="Azure Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/auth0-integration">
                        <img
                          src="/img/auth0.svg"
                          alt="Auth0 Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/Github">
                        <img
                          src="/img/github.svg"
                          alt="Github Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4 shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/Keycloak">
                        <img
                          src="/img/keycloak.png"
                          alt="Keycloak Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                    <div className="flex justify-center items-center w-full p-4  shadow-orange-200 border border-orange-200 hover:bg-orange-100">
                      <Link to="/docs/Integration/Identity%20providers/GCP">
                        <img
                          src="/img/gcp.png"
                          alt="GCP Logo"
                          className="h-32 object-contain"
                        />
                      </Link>
                    </div>
                  </div>
                </div>
                <div className="order-1 md:order-2 space-y-2 p-6">
                  <h3 className="text-5xl font-bold">
                    Support for authentication with{" "}
                    <Link to="/docs/Authentication/Intro">
                      <span className="text-blue-600 hover:text-blue-500 underline">
                        OIDC
                      </span>{" "}
                      and{" "}
                    </Link>
                    <Link to="/docs/Authentication/smart-on-fhir">
                      <span className="text-blue-600 hover:text-blue-500  underline">
                        SMART on FHIR
                      </span>
                    </Link>
                  </h3>
                  <div className="grid md:grid-cols-3 grid-cols-1 gap-4 mt-4 py-4">
                    <DescriptionColumn
                      title={
                        <Link
                          className="hover:text-blue-500"
                          to="/docs/category/openid-connect"
                        >
                          Grants
                        </Link>
                      }
                      description="Support for Authorization Code, Client Credentials, and Refresh Token grants."
                    />
                    <DescriptionColumn
                      title={
                        <Link
                          className="hover:text-blue-500"
                          to="/docs/Authentication/Federated%20Login"
                        >
                          Federated login
                        </Link>
                      }
                      description="Login with any identity provider that supports OIDC."
                    />
                    <DescriptionColumn
                      title={
                        <Link
                          className="hover:text-blue-500"
                          to="/docs/Authentication/Scopes"
                        >
                          Scopes
                        </Link>
                      }
                      description="Request only the FHIR resource access you need with fine-grained scopes."
                    />
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
