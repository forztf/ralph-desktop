<script lang="ts">
  import type { ProjectMeta, ProjectStatus } from '$lib/types';

  interface Props {
    projects: ProjectMeta[];
    selectedId: string | null;
    onSelect: (id: string) => void;
    onDelete: (id: string) => void;
  }

  let { projects, selectedId, onSelect, onDelete }: Props = $props();

  const statusConfig: Record<ProjectStatus, { icon: string; color: string; animate?: boolean }> = {
    brainstorming: { icon: '💭', color: 'text-purple-500' },
    ready: { icon: '⚪', color: 'text-gray-400' },
    queued: { icon: '🔵', color: 'text-blue-500' },
    running: { icon: '🟢', color: 'text-green-500', animate: true },
    pausing: { icon: '🟡', color: 'text-yellow-500', animate: true },
    paused: { icon: '🟡', color: 'text-yellow-500' },
    done: { icon: '✅', color: 'text-green-600' },
    failed: { icon: '❌', color: 'text-red-500' },
    cancelled: { icon: '🚫', color: 'text-gray-500' }
  };

  function getStatusInfo(status: ProjectStatus) {
    return statusConfig[status] || statusConfig.ready;
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
  }
</script>

<div class="divide-y divide-gray-100 dark:divide-gray-700">
  {#if projects.length === 0}
    <div class="p-4 text-center text-gray-500 dark:text-gray-400 text-sm">
      暂无项目
    </div>
  {:else}
    {#each projects as project (project.id)}
      {@const statusInfo = getStatusInfo(project.status)}
      <div
        class="w-full p-3 text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors group cursor-pointer
          {selectedId === project.id ? 'bg-blue-50 dark:bg-blue-900/30 border-l-2 border-blue-600' : ''}"
        onclick={() => onSelect(project.id)}
        onkeydown={(e) => e.key === 'Enter' && onSelect(project.id)}
        role="button"
        tabindex="0"
      >
        <div class="flex items-start gap-2">
          <span class="text-lg mt-0.5 {statusInfo.animate ? 'animate-pulse' : ''}">{statusInfo.icon}</span>
          <div class="flex-1 min-w-0">
            <div class="font-medium text-gray-800 dark:text-white truncate">
              {project.name}
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5">
              {project.path}
            </div>
            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">
              {formatDate(project.lastOpenedAt)}
            </div>
          </div>
          <button
            class="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
            onclick={(e) => { e.stopPropagation(); onDelete(project.id); }}
            title="删除项目"
          >
            ✕
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
