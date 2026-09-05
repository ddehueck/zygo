import { TokenFieldSegment } from "react-aria-components/TokenField";

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
    };

export type WorkflowRunFilter =
  | {
      entity: "workflow";
      id: string;
    }
  | {
      entity: "tag";
      name: string;
      value?: string;
    };

export type WorkflowSearchTokenSegment = TokenFieldSegment<WorkflowRunFilter>;
