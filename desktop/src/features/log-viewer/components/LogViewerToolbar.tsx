import { Button } from "@/components/Button";
import { Checkbox } from "@/components/Checkbox";

export function LogViewerToolbar({
  isFollowing,
  onFollowChange,
  onJumpToLatest,
}: {
  isFollowing: boolean;
  onFollowChange: (isFollowing: boolean) => void;
  onJumpToLatest: () => void;
}) {
  return (
    <div className="flex min-h-12 shrink-0 items-center justify-between gap-4 border-b border-app-border px-3 py-2">
      <Checkbox isSelected={isFollowing} onChange={onFollowChange}>
        Follow output
      </Checkbox>
      <Button variant="secondary" isDisabled={isFollowing} onPress={onJumpToLatest}>
        Jump to latest
      </Button>
    </div>
  );
}
