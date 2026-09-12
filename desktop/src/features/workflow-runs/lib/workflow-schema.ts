import { z } from "zod";

const channelShape = z.object({
  id: z.string(),
  accepted_file_extensions: z.array(z.string()).default([]),
});

const jobShape = z.object({
  id: z.string(),
  input_channel_id: z.string(),
  output_channel_id: z.string(),
});

const workflowSchemaShape = z.object({
  input_channel_id: z.string(),
  jobs: z.array(jobShape).default([]),
  channels: z.array(channelShape),
});

export type ParsedWorkflowSchema = z.infer<typeof workflowSchemaShape>;

export type WorkflowJobOption = {
  id: string;
  inputChannelId: string;
  acceptedExtensions: string[];
};

/**
 * Parses the minimal workflow schema fields needed by the new-run UI.
 */
export function parseWorkflowSchema(schemaJson: string): ParsedWorkflowSchema {
  return workflowSchemaShape.parse(JSON.parse(schemaJson));
}

function normalizeExtensions(extensions: string[]): string[] {
  return extensions.map((extension) => extension.replace(/^\.+/, ""));
}

function extensionsForChannel(
  schema: ParsedWorkflowSchema,
  channelId: string,
): string[] {
  const channel = schema.channels.find((entry) => entry.id === channelId);
  return normalizeExtensions(channel?.accepted_file_extensions ?? []);
}

/**
 * Reads accepted input file extensions for the full workflow, or for a single job
 * when `jobId` is provided (using that job's input channel).
 */
export function acceptedInputFileExtensions(
  schemaJson: string,
  jobId?: string | null,
): string[] {
  const schema = parseWorkflowSchema(schemaJson);

  if (!jobId) {
    return extensionsForChannel(schema, schema.input_channel_id);
  }

  const job = schema.jobs.find((entry) => entry.id === jobId);
  if (!job) {
    throw new Error(`Job "${jobId}" was not found in workflow schema.`);
  }

  return extensionsForChannel(schema, job.input_channel_id);
}

export function workflowJobOptions(schemaJson: string): WorkflowJobOption[] {
  const schema = parseWorkflowSchema(schemaJson);

  return schema.jobs.map((job) => ({
    id: job.id,
    inputChannelId: job.input_channel_id,
    acceptedExtensions: extensionsForChannel(schema, job.input_channel_id),
  }));
}
