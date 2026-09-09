import type { JobRun, Tag, TauriDataReference } from "@/bindings";

import { genCompareFn } from "@/lib/compare";
import { appendToMap } from "@/lib/map";

const compareReferences = genCompareFn<TauriDataReference>(
  (reference) => reference.created_at,
  (reference) => reference.id,
);

// Looks up refs jobs and tags for the data tree
export class DataTreeIndex {
  readonly referencesById: ReadonlyMap<number, TauriDataReference>;
  readonly jobsById: ReadonlyMap<number, JobRun>;
  readonly tagsByRefId: ReadonlyMap<number, readonly Tag[]>;
  readonly rootIds: readonly number[];

  private readonly jobsByCanonicalInputId: ReadonlyMap<number, readonly JobRun[]>;
  private readonly outputsByJobId: ReadonlyMap<number, readonly TauriDataReference[]>;

  constructor(references: TauriDataReference[], jobs: JobRun[], tags: Tag[] = []) {
    this.referencesById = new Map(references.map((reference) => [reference.id, reference]));
    this.jobsById = new Map(jobs.map((job) => [job.id, job]));
    this.tagsByRefId = buildTagsByRefId(tags);

    const canonicalIdByUri = buildCanonicalIdByUri(references);
    this.jobsByCanonicalInputId = buildJobsByCanonicalInputId(
      jobs,
      this.referencesById,
      canonicalIdByUri,
    );

    const { rootIds, outputsByJobId } = buildRootsAndJobOutputs(references);
    this.rootIds = rootIds;
    this.outputsByJobId = outputsByJobId;
  }

  childIds(parentId: number): number[] {
    const consumerJobs = this.jobsByCanonicalInputId.get(parentId) ?? [];
    return consumerJobs.flatMap((job) =>
      (this.outputsByJobId.get(job.id) ?? []).map((output) => output.id),
    );
  }

  compareIds(a: number, b: number): number {
    const left = this.referencesById.get(a);
    const right = this.referencesById.get(b);
    if (!left || !right) return a - b;
    return compareReferences(left, right);
  }
}

function buildTagsByRefId(tags: Tag[]): Map<number, Tag[]> {
  const tagsByRefId = new Map<number, Tag[]>();
  for (const tag of tags) {
    if (tag.data_reference_id === null) continue;
    appendToMap(tagsByRefId, tag.data_reference_id, tag);
  }
  return tagsByRefId;
}

// Prefer the job output when the same uri shows up twice
function buildCanonicalIdByUri(references: TauriDataReference[]): Map<string, number> {
  const canonicalIdByUri = new Map<string, number>();

  for (const reference of references) {
    if (reference.source_job_run_id === null && !canonicalIdByUri.has(reference.uri)) {
      canonicalIdByUri.set(reference.uri, reference.id);
    }
  }
  for (const reference of references) {
    if (reference.source_job_run_id !== null) {
      canonicalIdByUri.set(reference.uri, reference.id);
    }
  }

  return canonicalIdByUri;
}

function buildJobsByCanonicalInputId(
  jobs: JobRun[],
  referencesById: ReadonlyMap<number, TauriDataReference>,
  canonicalIdByUri: ReadonlyMap<string, number>,
): Map<number, JobRun[]> {
  const jobsByCanonicalInputId = new Map<number, JobRun[]>();

  for (const job of jobs) {
    const reference = referencesById.get(job.input_id);
    if (!reference) continue;
    const canonicalInputId = canonicalIdByUri.get(reference.uri) ?? reference.id;
    appendToMap(jobsByCanonicalInputId, canonicalInputId, job);
  }

  return jobsByCanonicalInputId;
}

// First null source row for a uri is a root later ones are not
function buildRootsAndJobOutputs(references: TauriDataReference[]): {
  rootIds: number[];
  outputsByJobId: Map<number, TauriDataReference[]>;
} {
  const earliestIdByUri = new Map<string, number>();
  for (const reference of references) {
    const earliest = earliestIdByUri.get(reference.uri);
    if (earliest === undefined || reference.id < earliest) {
      earliestIdByUri.set(reference.uri, reference.id);
    }
  }

  const rootIds: number[] = [];
  const outputsByJobId = new Map<number, TauriDataReference[]>();

  for (const reference of references) {
    if (reference.source_job_run_id !== null) {
      appendToMap(outputsByJobId, reference.source_job_run_id, reference);
      continue;
    }
    if (earliestIdByUri.get(reference.uri) === reference.id) {
      rootIds.push(reference.id);
    }
  }

  return { rootIds, outputsByJobId };
}
