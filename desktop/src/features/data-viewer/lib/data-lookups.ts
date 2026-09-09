import type { JobRun, TauriDataReference } from "@/bindings";

import { dummySizeBytes, formatByteSize } from "./dummy-metadata";
import { getFileKind, getFileNameFromUri } from "./file-name";

export type DataRowPresentation = {
  fileName: string;
  kind: string;
  sizeBytes: number;
  sizeLabel: string;
};

export function presentDataRow(reference: TauriDataReference): DataRowPresentation {
  const sizeBytes = dummySizeBytes(reference.id);
  return {
    fileName: getFileNameFromUri(reference.uri),
    kind: getFileKind(reference.uri),
    sizeBytes,
    sizeLabel: formatByteSize(sizeBytes),
  };
}

export function getProducedByJob(
  reference: TauriDataReference,
  jobsById: ReadonlyMap<number, JobRun>,
): JobRun | null {
  if (reference.source_job_run_id === null) return null;
  return jobsById.get(reference.source_job_run_id) ?? null;
}
