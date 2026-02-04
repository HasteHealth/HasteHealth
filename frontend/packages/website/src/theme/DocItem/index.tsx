import React, { type ReactNode } from "react";
import DocItem from "@theme-original/DocItem";
import type DocItemType from "@theme/DocItem";
import type { WrapperProps } from "@docusaurus/types";
import GiscusComponent from "@site/src/components/Comments";

type Props = WrapperProps<typeof DocItemType>;

export default function DocItemWrapper(props: Props): ReactNode {
  const { frontMatter } = props.content;
  // @ts-ignore
  const { enable_comments } = frontMatter;

  return (
    <>
      <DocItem {...props} />
      {enable_comments ? <GiscusComponent /> : null}
    </>
  );
}
