<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState, type Extension } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { javascript } from '@codemirror/lang-javascript';
  import { json } from '@codemirror/lang-json';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { python } from '@codemirror/lang-python';
  import { rust } from '@codemirror/lang-rust';
  import { markdown } from '@codemirror/lang-markdown';
  import { isDark } from '$lib/stores/theme.svelte';

  // Props
  let {
    content = $bindable(''),
    filename = '',
    pane = 'remote' as 'local' | 'remote',
    onSave,
    saving = false,
    loading = false
  }: {
    content: string;
    filename: string;
    pane: 'local' | 'remote';
    onSave: () => void;
    saving?: boolean;
    loading?: boolean;
  } = $props();

  // Large file guard: 5 MB
  const MAX_BYTES = 5 * 1024 * 1024;
  let byteSize = $derived(new TextEncoder().encode(content).length);
  let sizeWarningDismissed = $state(false);
  let showSizeWarning = $derived(byteSize > MAX_BYTES && !sizeWarningDismissed);

  // Status bar
  let line = $state(1);
  let col = $state(1);
  let saveStatus = $state<'idle' | 'saving' | 'saved'>('idle');

  $effect(() => {
    if (saving) saveStatus = 'saving';
  });

  // Notify saved briefly after saving transitions false
  let prevSaving = false;
  $effect(() => {
    if (prevSaving && !saving && saveStatus === 'saving') {
      saveStatus = 'saved';
      setTimeout(() => { saveStatus = 'idle'; }, 2500);
    }
    prevSaving = saving;
  });

  function getLanguage(name: string): Extension {
    const ext = name.split('.').pop()?.toLowerCase() ?? '';
    switch (ext) {
      case 'js':
      case 'mjs':
      case 'cjs':
        return javascript();
      case 'ts':
      case 'tsx':
        return javascript({ typescript: true });
      case 'jsx':
        return javascript({ jsx: true });
      case 'json':
      case 'jsonc':
        return json();
      case 'html':
      case 'htm':
      case 'svelte':
        return html();
      case 'css':
      case 'pcss':
      case 'scss':
        return css();
      case 'py':
        return python();
      case 'rs':
        return rust();
      case 'md':
      case 'mdx':
      case 'markdown':
        return markdown();
      default:
        return [];
    }
  }

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;

  function buildExtensions(dark: boolean): Extension[] {
    return [
      lineNumbers(),
      highlightActiveLine(),
      history(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      getLanguage(filename),
      dark ? oneDark : [],
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        indentWithTab,
        {
          key: 'Ctrl-s',
          mac: 'Cmd-s',
          run: () => { onSave(); return true; }
        }
      ]),
      EditorView.updateListener.of((update) => {
        // sync content out via bindable
        if (update.docChanged) {
          content = update.state.doc.toString();
        }
        // update cursor position for status bar
        const sel = update.state.selection.main;
        const pos = sel.head;
        const l = update.state.doc.lineAt(pos);
        line = l.number;
        col = pos - l.from + 1;
      }),
      EditorView.theme({
        '&': { height: '100%', fontSize: '12px' },
        '.cm-scroller': { overflow: 'auto', fontFamily: 'ui-monospace, monospace' }
      })
    ];
  }

  function initEditor() {
    if (!editorContainer) return;
    if (view) { view.destroy(); view = null; }

    const state = EditorState.create({
      doc: content,
      extensions: buildExtensions(isDark())
    });

    view = new EditorView({ state, parent: editorContainer });
  }

  // Reinit when theme changes
  $effect(() => {
    const dark = isDark();
    if (!view) return;
    view.destroy();
    view = null;
    const state = EditorState.create({
      doc: content,
      extensions: buildExtensions(dark)
    });
    view = new EditorView({ state, parent: editorContainer });
  });

  // Sync incoming content changes (e.g. fresh file load) into CM
  $effect(() => {
    if (!view) return;
    const current = view.state.doc.toString();
    if (content !== current) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: content }
      });
    }
  });

  onMount(() => {
    if (!showSizeWarning) initEditor();
  });

  onDestroy(() => {
    view?.destroy();
  });

  function dismissWarningAndLoad() {
    sizeWarningDismissed = true;
    // Give Svelte one tick to remove the warning overlay before mounting CM
    setTimeout(initEditor, 0);
  }

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="flex flex-col h-full w-full overflow-hidden">
  <!-- Large-file guard -->
  {#if showSizeWarning}
    <div class="flex-1 flex flex-col items-center justify-center gap-4 bg-neutral-50 dark:bg-[#12161b] text-xs p-6 text-center">
      <span class="text-3xl">⚠️</span>
      <p class="text-neutral-700 dark:text-slate-300 font-medium">
        This file is <span class="font-bold text-amber-600 dark:text-amber-400">{formatBytes(byteSize)}</span>,
        which exceeds the 5 MB safe-edit limit.
      </p>
      <p class="text-neutral-500 dark:text-slate-500">Loading large files may cause the editor to freeze or become sluggish.</p>
      <button
        onclick={dismissWarningAndLoad}
        class="px-4 py-2 bg-amber-500 hover:bg-amber-400 text-white rounded text-xs font-semibold"
      >
        Load anyway
      </button>
    </div>
  {:else}
    <!-- CodeMirror host -->
    <div
      bind:this={editorContainer}
      class="flex-1 overflow-hidden"
    ></div>
  {/if}

  <!-- Status bar -->
  <div class="flex items-center justify-between px-3 py-1 border-t border-neutral-200 dark:border-slate-800 bg-neutral-100 dark:bg-slate-900 text-[10px] text-neutral-500 dark:text-slate-500 select-none shrink-0">
    <span>Ln {line}, Col {col}</span>
    <span class="flex items-center gap-3">
      <span>UTF-8</span>
      <span class="uppercase text-neutral-400 dark:text-slate-600">{pane}</span>
      {#if loading}
        <span class="text-cyan-500">Loading…</span>
      {:else if saveStatus === 'saving'}
        <span class="text-amber-500">Saving…</span>
      {:else if saveStatus === 'saved'}
        <span class="text-emerald-500">✓ Saved</span>
      {/if}
    </span>
  </div>
</div>
