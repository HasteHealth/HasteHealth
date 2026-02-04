import React from "react";
import Giscus from "@giscus/react";

export default function GiscusComponent() {
  return (
    <Giscus
      repo="HasteHealth/HasteHealth"
      repoId="R_kgDOPQcQjg"
      category="General"
      categoryId="DIC_kwDOPQcQjs4C129j"
      //   categoryId="IdOfDiscussionCategory" // E.g. id of "General"
      mapping="url" // Important! To map comments to URL
      term="Welcome to @giscus/react component!"
      strict="0"
      reactionsEnabled="1"
      emitMetadata="1"
      inputPosition="top"
      theme={"light"}
      lang="en"
      loading="lazy"
    />
  );
}
