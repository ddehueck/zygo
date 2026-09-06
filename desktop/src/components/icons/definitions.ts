import { ChevronLeft, ChevronRight, FileText, Search, Tag, X, type LucideIcon } from "lucide-react";

export type OpticalContext = "inline" | "circle" | "square" | "pill";

type OpticalAdjustment = {
  x?: number;
  y?: number;
};

export type IconDefinition = {
  icon: LucideIcon;
  optical?: Partial<Record<OpticalContext, OpticalAdjustment>>;
};

export const iconDefinitions = {
  close: {
    icon: X,
  },
  file: {
    icon: FileText,
  },
  next: {
    icon: ChevronRight,
    optical: {
      circle: { x: 0.5 },
    },
  },
  previous: {
    icon: ChevronLeft,
    optical: {
      circle: { x: -0.5 },
    },
  },
  search: {
    icon: Search,
  },
  tag: {
    icon: Tag,
  },
} satisfies Record<string, IconDefinition>;
