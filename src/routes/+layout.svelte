<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { projects, currentProjectId, currentProject, updateProjects, updateCurrentProject, selectProject } from '$lib/stores/projects';
  import { config, availableClis, updateConfig, setAvailableClis } from '$lib/stores/settings';
  import { loopState, setStatus, setIteration, addLog, setError } from '$lib/stores/loop';
  import { notifySuccess, notifyError, notifyWarning } from '$lib/stores/notifications';
  import * as api from '$lib/services/tauri';
  import type { LoopEvent } from '$lib/types';
  import type { RecoveryInfo } from '$lib/services/tauri';
  import RecoveryDialog from '$lib/components/RecoveryDialog.svelte';
  import NotificationToast from '$lib/components/NotificationToast.svelte';

  let { children } = $props();
  let initialized = $state(false);
  let showPermissionDialog = $state(false);
  let showRecoveryDialog = $state(false);
  let interruptedTasks = $state<RecoveryInfo[]>([]);

  onMount(async () => {
    try {
      // Load config
      const loadedConfig = await api.getConfig();
      updateConfig(loadedConfig);

      // Check if permissions need confirmation
      if (!loadedConfig.permissionsConfirmed) {
        showPermissionDialog = true;
      }

      // Detect CLIs
      const clis = await api.detectInstalledClis();
      setAvailableClis(clis);

      // Load projects
      const projectList = await api.listProjects();
      updateProjects(projectList);

      // Check for interrupted tasks
      const interrupted = await api.checkInterruptedTasks();
      if (interrupted.length > 0) {
        interruptedTasks = interrupted;
        showRecoveryDialog = true;
      }

      // Listen to loop events
      await api.listenToLoopEvents(handleLoopEvent);

      // Clean up old logs on startup
      await api.cleanupLogs();

      initialized = true;
    } catch (error) {
      console.error('Initialization error:', error);
      initialized = true; // Still show UI even on error
    }
  });

  function handleLoopEvent(event: LoopEvent) {
    const statusMap: Record<string, string> = {
      iterationStart: 'running',
      pausing: 'pausing',
      paused: 'paused',
      resumed: 'running',
      completed: 'done',
      maxIterationsReached: 'failed',
      stopped: 'cancelled'
    };

    if (event.type === 'output' && event.content) {
      addLog({
        iteration: event.iteration || 0,
        content: event.content,
        isStderr: event.isStderr || false,
        timestamp: new Date()
      });
    }

    if (event.type === 'error' && event.error) {
      setError(event.error);
      notifyError('执行错误', event.error);
    }

    if (event.type === 'completed') {
      notifySuccess('任务完成', `项目已成功完成所有迭代`);
    }

    if (event.type === 'maxIterationsReached') {
      notifyWarning('达到最大迭代次数', `任务已在第 ${event.iteration} 次迭代后停止`);
    }

    if (event.iteration !== undefined) {
      setIteration(event.iteration);
    }

    const newStatus = statusMap[event.type];
    if (newStatus) {
      setStatus(newStatus as any);
    }
  }

  async function handleConfirmPermissions() {
    await api.confirmPermissions();
    const loadedConfig = await api.getConfig();
    updateConfig(loadedConfig);
    showPermissionDialog = false;
  }

  function handleRecoverTask(projectId: string) {
    selectProject(projectId);
    interruptedTasks = interruptedTasks.filter(t => t.projectId !== projectId);
    if (interruptedTasks.length === 0) {
      showRecoveryDialog = false;
    }
  }

  function handleCancelTask(projectId: string) {
    interruptedTasks = interruptedTasks.filter(t => t.projectId !== projectId);
    if (interruptedTasks.length === 0) {
      showRecoveryDialog = false;
    }
  }

  function handleDismissRecovery() {
    showRecoveryDialog = false;
  }
</script>

{#if showPermissionDialog}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg p-6 m-4">
      <h2 class="text-xl font-bold text-amber-600 dark:text-amber-400 mb-4">
        ⚠️ 重要安全提示
      </h2>
      <div class="text-gray-700 dark:text-gray-300 space-y-3 mb-6">
        <p>Ralph Desktop 需要以自动执行模式运行 AI 编程助手。这意味着：</p>
        <ul class="list-disc list-inside space-y-1 ml-2">
          <li>AI 可以自动读取、创建、修改、删除项目目录中的文件</li>
          <li>AI 可以自动执行命令（如 npm install、git commit 等）</li>
          <li>AI 的操作不会逐一询问确认</li>
        </ul>
        <p class="font-medium">建议：</p>
        <ul class="list-disc list-inside space-y-1 ml-2">
          <li>仅在你信任的项目目录中使用</li>
          <li>确保项目使用 Git 版本控制，以便回滚</li>
          <li>不要在包含敏感数据的目录中使用</li>
        </ul>
      </div>
      <div class="flex justify-end gap-3">
        <button
          class="px-4 py-2 rounded-lg bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600"
          onclick={() => window.close()}
        >
          取消
        </button>
        <button
          class="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
          onclick={handleConfirmPermissions}
        >
          确认并继续
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showRecoveryDialog && interruptedTasks.length > 0}
  <RecoveryDialog
    tasks={interruptedTasks}
    onRecover={handleRecoverTask}
    onCancel={handleCancelTask}
    onDismiss={handleDismissRecovery}
  />
{/if}

{#if initialized}
  {@render children()}
{:else}
  <div class="flex items-center justify-center h-screen bg-gray-100 dark:bg-gray-900">
    <div class="text-center">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
      <p class="text-gray-600 dark:text-gray-400">加载中...</p>
    </div>
  </div>
{/if}

<NotificationToast />
