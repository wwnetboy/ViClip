export function HoverProgress({ progress }: { progress: number }) {
  return (
    <div className="hover-progress-track">
      <div
        className="hover-progress-bar"
        style={{ width: `${progress * 100}%` }}
      />
    </div>
  );
}
