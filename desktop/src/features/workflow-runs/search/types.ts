export type WorkflowSearchValue = "workflow" | "tag";

export type WorkflowRunSearchSuggestion =
  | {
      id: string;
      type: "prefix";
      text: string;
    }
  | {
      id: string;
      type: "token";
      text: string;
      value: WorkflowSearchValue;
    };
