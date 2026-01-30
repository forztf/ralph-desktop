<script lang="ts">
  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show, onClose }: Props = $props();

  const shortcuts = [
    { keys: ['⌘', 'N'], description: '新建项目' },
    { keys: ['⌘', ','], description: '打开设置' },
    { keys: ['⌘', 'Enter'], description: '开始/继续任务' },
    { keys: ['⌘', 'P'], description: '暂停任务' },
    { keys: ['⌘', '.'], description: '停止任务' },
    { keys: ['Esc'], description: '关闭对话框' },
    { keys: ['↑', '↓'], description: '切换项目' },
    { keys: ['⌘', '?'], description: '显示快捷键帮助' },
  ];
</script>

{#if show}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onclick={onClose}>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full m-4"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold text-gray-800 dark:text-white">键盘快捷键</h2>
        <button
          class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-gray-500"
          onclick={onClose}
        >
          ✕
        </button>
      </div>
      <div class="p-4">
        <div class="space-y-3">
          {#each shortcuts as shortcut}
            <div class="flex items-center justify-between">
              <span class="text-gray-700 dark:text-gray-300">{shortcut.description}</span>
              <div class="flex gap-1">
                {#each shortcut.keys as key}
                  <kbd class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-gray-700 dark:text-gray-300">
                    {key}
                  </kbd>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </div>
      <div class="p-4 border-t border-gray-200 dark:border-gray-700">
        <p class="text-xs text-gray-500 dark:text-gray-400 text-center">
          按 <kbd class="px-1.5 py-0.5 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded">Esc</kbd> 关闭
        </p>
      </div>
    </div>
  </div>
{/if}
