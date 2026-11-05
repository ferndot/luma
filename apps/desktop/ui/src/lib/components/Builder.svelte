<script lang="ts">
  import type { NamedEffect, Scene } from '../api'
  import EffectBuilder from './EffectBuilder.svelte'
  import SceneBuilder from './SceneBuilder.svelte'

  let {
    effects,
    scenes,
    activeEffect,
    activeScene,
    onsaveeffect,
    onreseteffect,
    ondeleteeffect,
    onplayeffect,
    onsavescene,
    ondeletescene,
    onplayscene,
  }: {
    effects: NamedEffect[]
    scenes: Scene[]
    activeEffect: string | null
    activeScene: string | null
    onsaveeffect: (effect: NamedEffect) => Promise<string | null>
    onreseteffect: (id: string) => void
    ondeleteeffect: (id: string) => void
    onplayeffect: (id: string) => void
    onsavescene: (scene: Scene) => Promise<string | null>
    ondeletescene: (id: string) => void
    onplayscene: (id: string) => void
  } = $props()

  let tab = $state<'effects' | 'scenes'>('effects')
</script>

<div class="builder">
  <nav class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'effects'} class:active={tab === 'effects'} onclick={() => (tab = 'effects')}>Effects</button>
    <button role="tab" aria-selected={tab === 'scenes'} class:active={tab === 'scenes'} onclick={() => (tab = 'scenes')}>Scenes</button>
  </nav>
  {#if tab === 'effects'}
    <EffectBuilder {effects} {activeEffect} onsave={onsaveeffect} onreset={onreseteffect} ondelete={ondeleteeffect} onplay={onplayeffect} />
  {:else}
    <SceneBuilder {scenes} {effects} {activeScene} onsave={onsavescene} ondelete={ondeletescene} onplay={onplayscene} />
  {/if}
</div>

<style>
  .builder {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .tabs {
    display: flex;
    gap: 4px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
  }
  .tabs button {
    padding: 10px 14px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .tabs button.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
</style>
