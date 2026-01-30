<script lang="ts">
  import { queueStatus, projectQueue, runningProjects } from '$lib/stores/queue';

  const statusText = $derived(() => {
    const status = $queueStatus;
    if (status.runningCount === 0 && status.queueLength === 0) {
      return '空闲';
    }
    let text = '';
    if (status.runningCount > 0) {
      text += `${status.runningCount} 个运行中`;
    }
    if (status.queueLength > 0) {
      if (text) text += ', ';
      text += `${status.queueLength} 个排队中`;
    }
    return text;
  });

  const statusColor = $derived(() => {
    const status = $queueStatus;
    if (status.runningCount > 0) return 'text-green-500';
    if (status.queueLength > 0) return 'text-blue-500';
    return 'text-gray-400';
  });
</script>

<div class="flex items-center gap-2 px-3 py-2 bg-gray-50 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
  <div class="flex items-center gap-1.5">
    <div class="w-2 h-2 rounded-full {$queueStatus.runningCount > 0 ? 'bg-green-500 animate-pulse' : 'bg-gray-300'}"></div>
    <span class="text-xs {statusColor()}">{statusText()}</span>
  </div>
  {#if $queueStatus.availableSlots > 0}
    <span class="text-xs text-gray-400 ml-auto">
      {$queueStatus.availableSlots} 槽位可用
    </span>
  {/if}
</div>
