# Generated Operations

Generated Operation classes based on OperationDefintions found in load-artifacts (covers hl7 and oxidized-health operations).
Provides utility methods to validate parameters and parses parameters from simple javascript maps using internally stored OperationDefinition.

## Usage

```typescript
import httpClient from "@oxidized-health/client/http";
import { ValueSetExpand } from "./ops.js";

const client = httpClient(configuration);
const response  = await client.invoke_system(ValueSetExpand.Op, ctx, { url: "value-set-url" }))
```
