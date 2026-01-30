<script lang="ts">
  import type { LogEntry } from '$lib/types';

  interface Props {
    logs: LogEntry[];
  }

  let { logs }: Props = $props();
  let container: HTMLDivElement;
  let autoScroll = $state(true);

  // Auto-scroll to bottom when new logs arrive
  $effect(() => {
    if (logs.length && autoScroll && container) {
      container.scrollTop = container.scrollHeight;
    }
  });

  function handleScroll() {
    if (container) {
      const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;
      autoScroll = isAtBottom;
    }
  }

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
</script>

<div
  bind:this={container}
  class="h-full overflow-y-auto font-mono text-sm p-4"
  onscroll={handleScroll}
>
  {#if logs.length === 0}
    <div class="text-gray-500 text-center py-8">
      等待日志输出...
    </div>
  {:else}
    {#each logs as log, i (i)}
      <div class="flex gap-2 hover:bg-gray-800/50 py-0.5 {log.isStderr ? 'text-red-400' : 'text-green-400'}">
        <span class="text-gray-600 shrink-0">[#{log.iteration}]</span>
        <span class="text-gray-500 shrink-0">{formatTime(log.timestamp)}</span>
        <span class="break-all">{log.content}</span>
      </div>
    {/each}
  {/if}

  {#if !autoScroll && logs.length > 0}
    <button
      class="fixed bottom-20 right-8 px-3 py-1 bg-blue-600 text-white rounded-full text-xs shadow-lg hover:bg-blue-700"
      onclick={() => {
        autoScroll = true;
        container.scrollTop = container.scrollHeight;
      }}
    >
      ↓ 滚动到底部
    </button>
  {/if}
</div>
