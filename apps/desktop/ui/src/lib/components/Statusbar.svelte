<script lang="ts">
  type Tone = 'idle' | 'busy' | 'live' | 'error'

  let { text, tone }: { text: string; tone: Tone } = $props()
</script>

<footer class="statusbar {tone}" role="status" aria-live="polite">
  <span class="dot" aria-hidden="true"></span>
  <span class="text">{text}</span>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 26px;
    padding: 0 12px;
    border-top: 1px solid var(--line);
    background: var(--surface);
    color: var(--muted);
    font-size: 12px;
    user-select: none;
  }
  .text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #6b6580;
    flex: none;
  }
  .statusbar.live { color: var(--text); }
  .statusbar.live .dot {
    background: #22c55e;
    box-shadow: 0 0 6px #22c55e;
  }
  .statusbar.busy .dot {
    background: #facc15;
    animation: blink 1s ease-in-out infinite;
  }
  .statusbar.error { color: #fca5a5; }
  .statusbar.error .dot { background: #ef4444; }

  @keyframes blink {
    50% { opacity: 0.3; }
  }
</style>
