export interface PackageJSON {
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
}

export type IndexedFiles = {
  filename?: string;
  resourceType?: string;
  id?: string;
  url?: string;
  version?: string;
  kind?: string;
  type?: string;
  supplements?: string;
  content?: string;
};

export type IndexFile = {
  "index-version"?: string;
  files?: Array<IndexedFiles>;
};
