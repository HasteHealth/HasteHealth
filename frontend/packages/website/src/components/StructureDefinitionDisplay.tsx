import React from "react";
import { sdTraversal } from "@haste-health/codegen";
import {
  ElementDefinition,
  StructureDefinition,
} from "@haste-health/fhir-types/lib/generated/r4/types";

function isRequred(element: ElementDefinition): boolean {
  const min = element.min;
  return min !== undefined && min > 0;
}

function requiredIndicator(element: ElementDefinition): string {
  return isRequred(element) ? (
    <>
      <div className="border-b flex-grow ml-2" />{" "}
      <div className="text-red-600 ml-2">Required</div>
    </>
  ) : (
    <></>
  );
}

function isTypeChoice(element: ElementDefinition): boolean {
  return (element.type ?? []).length > 1;
}

function DisplayIfArray(element: ElementDefinition): string {}

function DisplayType({ element }: { element: ElementDefinition }) {
  const max = element.max ?? "1";

  const display = isTypeChoice(element)
    ? "typechoice"
    : (element.type ?? []).map((t) => t.code).join(", ");
  return (
    <div className="ml-2">
      <span
        className={`text-md font-semibold ${getColorCode(
          (element.type ?? [])[0]?.code ?? ""
        )}`}
      >
        {display}
        {max !== "1" ? " []" : ""}
      </span>
    </div>
  );
}

function getColorCode(typeCode: string): string {
  switch (typeCode) {
    case "string":
    case "markdown":
    case "uri":
    case "url":
    case "canonical":
      return "text-orange-600";

    case "boolean":
    case "integer":
    case "decimal":
      return "text-purple-600";
    case "code":
    case "Coding":
    case "CodeableConcept":
    case "Identifier":
      return "text-red-600";
    case "Quantity":
    case "Money":
      return "text-yellow-600";
    case "Reference":
      return "text-blue-600";
    case "date":
    case "dateTime":
    case "instant":
      return "text-indigo-600";
    default:
      return "text-slate-500";
  }
}

function SchemaItem({
  element,
  nested,
  children,
}: {
  nested: boolean;
  element: ElementDefinition;
  children: React.ReactNode;
}) {
  const [isActive, setIsActive] = React.useState<boolean>(false);
  const propertyDescription = (
    <>
      <summary
        className={`flex items-center font-semibold text-md ${
          !nested ? "cursor-default" : "cursor-pointer"
        }`}
      >
        <span className="font-bold">{element.path.split(".").pop()}</span>
        <DisplayType element={element} />
        {requiredIndicator(element)}
      </summary>
      <div className="">
        <span className="text-xs text-orange-900">{element.short}</span>
      </div>
    </>
  );

  if (!nested) {
    return <div className="schema-item w-full">{propertyDescription}</div>;
  }

  return (
    <div
      className="schema-item w-full"
      onClick={(_) => setIsActive((active) => !active)}
    >
      <div
        className="schema-item__details"
        data-collapsed={!isActive ? "true" : "false"}
      >
        {propertyDescription}
        <div
          style={{
            display: isActive ? "block" : "none",
            overflow: "hidden",
            height: isActive ? "auto" : "0px",
            willChange: "height",
            transition: "height 290ms ease-in-out",
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}

export default function StructureDefinitionDisplay(props: {
  sd: StructureDefinition;
}) {
  return sdTraversal.traversalBottomUp(
    props.sd,
    (
      element: ElementDefinition,
      nestedElements: React.JSX.Element[],
      { curIndex }
    ) => {
      if (curIndex == 0) {
        return <div>{nestedElements}</div>;
      } else {
        return (
          <SchemaItem nested={nestedElements.length > 0} element={element}>
            {nestedElements}
          </SchemaItem>
        );
      }
    }
  );
}
