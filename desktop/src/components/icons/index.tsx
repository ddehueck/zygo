import { type LucideProps } from "lucide-react";
import { StatusIcon, type StatusIconProps } from "./StatusIcon";

const statusIcon = (status: StatusIconProps["status"]) => (props: LucideProps) => (
  <StatusIcon status={status} {...props} />
);

export const Icons = {
  Completed: statusIcon("completed"),
  InProgress: statusIcon("in-progress"),
  Errored: statusIcon("errored"),
};

export { StatusIcon };
export type { StatusIconProps, StatusIconStatus } from "./StatusIcon";
