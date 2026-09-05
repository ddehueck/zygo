import type { LucideProps } from "lucide-react";
import type { IconDefinition, OpticalContext } from "./definitions";

type IconProps = LucideProps & {
  definition: IconDefinition;
  optical?: OpticalContext;
};

export function Icon({ definition, optical, style, ...props }: IconProps) {
  const Component = definition.icon;
  const adjustment = optical ? definition.optical?.[optical] : undefined;
  const transform = adjustment
    ? `translate(${adjustment.x ?? 0}px, ${adjustment.y ?? 0}px)`
    : undefined;

  return (
    <Component
      {...props}
      style={{
        ...style,
        transform: [transform, style?.transform].filter(Boolean).join(" ") || undefined,
      }}
    />
  );
}
