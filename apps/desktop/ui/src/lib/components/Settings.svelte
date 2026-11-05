<script lang="ts">
  let {
    hasKey,
    keyStatus,
    onsave,
  }: {
    hasKey: boolean
    keyStatus: string
    onsave: (key: string) => Promise<boolean>
  } = $props()

  let key = $state('')
  let showKey = $state(false)

  // Saving an empty value removes the key.
  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (await onsave(key)) key = ''
  }
</script>

<section class="settings">
  <h2 class="section-title">Settings</h2>

  <div class="group">
    <h3>Govee API key</h3>
    <p class="muted">
      Lets Luma read each device's name from your Govee account. Create a key in the Govee Home app under
      Settings, then Apply for API Key. The key is stored only on this computer.
    </p>

    <form class="key-row" onsubmit={submit}>
      <input
        type={showKey ? 'text' : 'password'}
        bind:value={key}
        placeholder={hasKey ? 'Key saved. Enter a new one to replace it' : 'Paste your API key'}
        aria-label="Govee API key"
        autocomplete="off"
        spellcheck="false"
      />
      <button type="button" class="ghost" onclick={() => (showKey = !showKey)}>
        {showKey ? 'Hide' : 'Show'}
      </button>
      <button type="submit" disabled={!key.trim()}>Save</button>
    </form>

    <div class="row">
      <p class="muted">{hasKey ? 'A key is saved.' : 'No key saved.'} {keyStatus}</p>
      {#if hasKey}
        <button class="ghost" onclick={() => onsave('')}>Remove key</button>
      {/if}
    </div>
  </div>
</section>

<style>
  .settings {
    padding: 16px;
    max-width: 640px;
    user-select: text;
  }
  .settings .section-title {
    margin-bottom: 16px;
  }
  .group {
    display: grid;
    gap: 8px;
  }
  h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .key-row {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }
  .key-row input {
    flex: 1;
    min-width: 0;
    padding: 7px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-family: ui-monospace, 'Cascadia Mono', Consolas, monospace;
  }
  .key-row button,
  .row button {
    padding: 6px 12px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
  }
  .key-row button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .ghost {
    background: transparent !important;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .row p {
    margin: 0;
  }
</style>
