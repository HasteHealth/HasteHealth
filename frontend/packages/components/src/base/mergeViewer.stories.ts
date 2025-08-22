import type { Meta, StoryObj } from "@storybook/react";

import { MergeViewer } from "./mergeViewer";

const meta = {
  title: "Base/Merge",
  component: MergeViewer,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof MergeViewer>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    oldValue: `{
      "id": "91a9eee0-34bd-4e36-8335-1631b6dec24b",
      "meta": {
        "extension": [
          {
            "url": "https://oxidized-health.app/version-sequence",
            "valueInteger": 4993
          },
          {
            "url": "https://oxidized-health.app/author",
            "valueString": "public-user"
          }
        ],
        "versionId": "4993",
        "lastUpdated": "2023-10-26T00:22:26.647+00:00"
      },
      "name": [
        {
          "text": "Mark Miller",
          "given": [
            "Mark"
          ],
          "family": "Miller"
        }
      ],
      "active": true,
      "gender": "female",
      "birthDate": "1990-10-22",
      "resourceType": "Patient"
    }`,
    newValue: `{
      "id": "newId",
      "meta": {
        "extension": [
          {
            "url": "https://oxidized-health.app/version-sequence",
            "valueInteger": 1942
          },
          {
            "url": "https://oxidized-health.app/author",
            "valueString": "public-user"
          }
        ],
        "versionId": "1942",
        "lastUpdated": "2023-10-26T00:19:39.730+00:00"
      },
      "name": [
        {
          "text": "Mark Miller",
          "given": [
            "Mark"
          ],
          "family": "Miller"
        }
      ],
      "active": true,
      "gender": "female",
      "birthDate": "1990-10-22",
      "resourceType": "Patient"
    }`,
  },
};
