import type { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import Heading from "@theme/Heading";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header
      className={clsx("hero hero--primary", styles.heroBanner)}
      style={{
        backgroundColor: "transparent",
      }}
    >
      <div className="container">
        <Heading as="h1" className="hero__title " style={{ color: "black" }}>
          {siteConfig.title}
        </Heading>
        {/* <p className="hero__subtitle text--secondary">{siteConfig.tagline}</p> */}
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro"
          >
            Tutorial - 5min ⏱️
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Haste Health`}
      description="Description will go into a meta tag in <head />"
    >
      <div
        style={{
          backgroundImage: "url(/img/swift.jpeg)",
          backgroundSize: "cover",
          height: "40vh",
          position: "absolute",
          width: "100%",
          backgroundPosition: "center",
        }}
      ></div>
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
