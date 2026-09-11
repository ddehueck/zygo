import { z } from "zod";

// Minimal schema shape needed to derive input-channel file filters.
const workflowSchemaInputShape = z.object({
  input_channel_id: z.string(),
  channels: z.array(
    z.object({
      id: z.string(),
      accepted_file_extensions: z.array(z.string()).default([]),
    }),
  ),
});

/**
 * Reads accepted input file extensions from a serialized workflow schema.
 * Extensions are normalized without a leading dot, matching `FileExtension`.
 */
export function acceptedInputFileExtensions(schemaJson: string): string[] {
  const schema = workflowSchemaInputShape.parse(JSON.parse(schemaJson));
  const inputChannel = schema.channels.find((channel) => channel.id === schema.input_channel_id);

  return (inputChannel?.accepted_file_extensions ?? []).map((extension) =>
    extension.replace(/^\.+/, ""),
  );
}
