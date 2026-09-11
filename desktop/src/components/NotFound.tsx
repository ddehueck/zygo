import { Description, Heading } from "./Text";

type NotFoundProps = {
  title: string;
  description: string;
};

export function NotFound({ title, description }: NotFoundProps) {
  return (
    <main className="mx-auto w-full max-w-5xl px-6 py-10">
      <Heading size="medium">{title}</Heading>
      <Description className="mt-2">{description}</Description>
    </main>
  );
}
