import type { ErrorComponentProps } from "@tanstack/react-router";
import { AppLayout } from "./layout/AppLayout";
import { Description, Heading, Text } from "./Text";
import { Highlightable } from "./Highlightable";

export function ReactError({ error }: ErrorComponentProps) {
  const message = error instanceof Error ? error.message : String(error);

  return (
    <AppLayout>
      <div className="mx-auto flex min-h-0 flex-1 items-center justify-center px-6 py-8">
        <Highlightable
          className="flex max-w-2xl flex-col items-center gap-3 text-center"
          role="alert"
          aria-live="assertive"
        >
          <Heading size="medium">Something went wrong</Heading>
          <Description>
            <Text className="wrap-break-word whitespace-pre-wrap">{message}</Text>
          </Description>
        </Highlightable>
      </div>
    </AppLayout>
  );
}
