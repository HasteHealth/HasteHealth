import { Navigate, useParams } from "react-router-dom";
import Projects from "./Projects";
import Users from "./Users";
import IdentityProviders from "./IdentityProviders";

export default function SystemResources() {
  const params = useParams();

  switch (params.resourceType) {
    case "User": {
      return <Users />;
    }
    case "IdentityProvider": {
      return <IdentityProviders />;
    }
    case "Project": {
      return <Projects />;
    }
    default: {
      return undefined;
    }
  }
}
