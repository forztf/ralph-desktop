<script lang="ts">
  import type { ProjectState } from '$lib/types';

  interface Props {
    project: ProjectState;
    onSave?: (prompt: string) => void;
    onCancel?: () => void;
  }

  let { project, onSave, onCancel }: Props = $props();

  let editedPrompt = $state(project.task?.prompt || '');
  let isEditing = $state(false);

  function handleEdit() {
    isEditing = true;
  }

  function handleSave() {
    onSave?.(editedPrompt);
    isEditing = false;
  }

  function handleCancel() {
    editedPrompt = project.task?.prompt || '';
    isEditing = false;
    onCancel?.();
  }

  function copyToClipboard() {
    navigator.clipboard.writeText(editedPrompt);
  }
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
  <!-- Header -->
  <div class="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700">
    <h3 class="font-medium text-gray-800 dark:text-white flex items-center gap-2">
      <span>📝</span>
      <span>Prompt 预览</span>
    </h3>
    <div class="flex gap-2">
      {#if !isEditing}
        <button
          class="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded text-gray-700 dark:text-gray-300"
          onclick={copyToClipboard}
        >
          复制
        </button>
        <button
          class="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900/30 hover:bg-blue-200 dark:hover:bg-blue-900/50 rounded text-blue-700 dark:text-blue-400"
          onclick={handleEdit}
        >
          编辑
        </button>
      {:else}
        <button
          class="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded text-gray-700 dark:text-gray-300"
          onclick={handleCancel}
        >
          取消
        </button>
        <button
          class="px-3 py-1 text-sm bg-green-600 hover:bg-green-700 rounded text-white"
          onclick={handleSave}
        >
          保存
        </button>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div class="p-3">
    {#if isEditing}
      <textarea
        class="w-full h-64 p-3 font-mono text-sm bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-gray-800 dark:text-gray-200"
        bind:value={editedPrompt}
        placeholder="输入 prompt..."
      ></textarea>
    {:else}
      <div class="max-h-64 overflow-y-auto">
        <pre class="whitespace-pre-wrap font-mono text-sm text-gray-700 dark:text-gray-300 bg-gray-50 dark:bg-gray-900 p-3 rounded-lg">{editedPrompt || '(无 prompt)'}</pre>
      </div>
    {/if}
  </div>

  <!-- Stats -->
  <div class="px-3 pb-3">
    <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-4">
      <span>字符数: {editedPrompt.length}</span>
      <span>行数: {editedPrompt.split('\n').length}</span>
    </div>
  </div>
</div>
