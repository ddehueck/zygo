import { Edge as SchemaEdge, Job, Channel } from "@/bindings";

type NodeWithOrder = {
  id: string;
  type: "job" | "channel";
  order: number;
};

/**
 * Computes the topological order of workflow nodes.
 *
 * The ordering follows data flow:
 * 1. Source channels (channels that no job outputs to) are first
 * 2. Jobs that receive from those channels come next
 * 3. Channels that those jobs output to come next
 * 4. And so on...
 *
 * @param jobs - Array of jobs in the workflow
 * @param channels - Array of channels in the workflow
 * @param edges - Array of edges connecting jobs and channels
 * @returns Array of node IDs with their computed order
 */
export function computeNodeOrder(
  jobs: Job[],
  channels: Channel[],
  edges: SchemaEdge[]
): NodeWithOrder[] {
  // Build adjacency maps based on edge kind
  // Input edge: channel -> job (job receives from channel)
  // Output edge: job -> channel (job sends to channel)
  const channelToJobs = new Map<string, string[]>(); // channels that feed into jobs
  const jobToChannels = new Map<string, string[]>(); // channels that jobs output to
  const channelHasInput = new Set<string>(); // channels that receive output from a job

  for (const edge of edges) {
    if (edge.kind === "input") {
      // Job receives from channel
      const jobList = channelToJobs.get(edge.channel_id) || [];
      jobList.push(edge.job_id);
      channelToJobs.set(edge.channel_id, jobList);
    } else if (edge.kind === "output") {
      // Job outputs to channel
      const channelList = jobToChannels.get(edge.job_id) || [];
      channelList.push(edge.channel_id);
      jobToChannels.set(edge.job_id, channelList);
      channelHasInput.add(edge.channel_id);
    }
  }

  // Find source channels (channels that no job outputs to)
  const sourceChannels = channels.filter((c) => !channelHasInput.has(c.id));

  const result: NodeWithOrder[] = [];
  const visited = new Set<string>();
  let currentOrder = 0;

  // BFS through the graph starting from source channels
  let currentChannels = sourceChannels.map((c) => c.id);

  while (
    currentChannels.length > 0 ||
    visited.size < jobs.length + channels.length
  ) {
    // Add current channels to result
    for (const channelId of currentChannels) {
      if (!visited.has(channelId)) {
        visited.add(channelId);
        result.push({ id: channelId, type: "channel", order: currentOrder });
      }
    }
    currentOrder++;

    // Find jobs that receive from current channels
    const nextJobs: string[] = [];
    for (const channelId of currentChannels) {
      const jobIds = channelToJobs.get(channelId) || [];
      for (const jobId of jobIds) {
        if (!visited.has(jobId)) {
          nextJobs.push(jobId);
        }
      }
    }

    // Add jobs to result
    const uniqueJobs = [...new Set(nextJobs)];
    for (const jobId of uniqueJobs) {
      if (!visited.has(jobId)) {
        visited.add(jobId);
        result.push({ id: jobId, type: "job", order: currentOrder });
      }
    }
    currentOrder++;

    // Find channels that current jobs output to
    const nextChannels: string[] = [];
    for (const jobId of uniqueJobs) {
      const channelIds = jobToChannels.get(jobId) || [];
      for (const channelId of channelIds) {
        if (!visited.has(channelId)) {
          nextChannels.push(channelId);
        }
      }
    }

    currentChannels = [...new Set(nextChannels)];

    // Handle disconnected nodes - if we're stuck but have unvisited nodes
    if (
      currentChannels.length === 0 &&
      visited.size < jobs.length + channels.length
    ) {
      // Find any unvisited channel to continue
      const unvisitedChannel = channels.find((c) => !visited.has(c.id));
      if (unvisitedChannel) {
        currentChannels = [unvisitedChannel.id];
      } else {
        // Find any unvisited job
        const unvisitedJob = jobs.find((j) => !visited.has(j.id));
        if (unvisitedJob) {
          visited.add(unvisitedJob.id);
          result.push({
            id: unvisitedJob.id,
            type: "job",
            order: currentOrder,
          });
          currentOrder++;
        }
      }
    }
  }

  return result;
}

/**
 * Creates a map from node ID to its topological order.
 */
export function createOrderMap(
  jobs: Job[],
  channels: Channel[],
  edges: SchemaEdge[]
): Map<string, number> {
  const orderedNodes = computeNodeOrder(jobs, channels, edges);
  const orderMap = new Map<string, number>();
  for (const node of orderedNodes) {
    orderMap.set(node.id, node.order);
  }
  return orderMap;
}
