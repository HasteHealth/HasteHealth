import { useParams } from "react-router-dom";
import Projects from "./Projects";
import ResourceTypes from "./ResourceTypes";

export default function SystemResources() {
  const params = useParams();

  switch (params.resourceType) {
    case "Project": {
      return <Projects />;
    }
    default: {
      return <ResourceTypes />;
    }
  }
}
