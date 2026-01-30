<script lang="ts">
  import { notifications, removeNotification, type Notification } from '$lib/stores/notifications';

  function getIcon(type: Notification['type']): string {
    switch (type) {
      case 'success': return '✅';
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
    }
  }

  function getBgColor(type: Notification['type']): string {
    switch (type) {
      case 'success': return 'bg-green-100 dark:bg-green-900/30 border-green-500';
      case 'error': return 'bg-red-100 dark:bg-red-900/30 border-red-500';
      case 'warning': return 'bg-yellow-100 dark:bg-yellow-900/30 border-yellow-500';
      case 'info': return 'bg-blue-100 dark:bg-blue-900/30 border-blue-500';
    }
  }

  function getTextColor(type: Notification['type']): string {
    switch (type) {
      case 'success': return 'text-green-800 dark:text-green-200';
      case 'error': return 'text-red-800 dark:text-red-200';
      case 'warning': return 'text-yellow-800 dark:text-yellow-200';
      case 'info': return 'text-blue-800 dark:text-blue-200';
    }
  }
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
  {#each $notifications as notification (notification.id)}
    <div
      class="flex items-start gap-3 p-3 rounded-lg border-l-4 shadow-lg animate-slide-in {getBgColor(notification.type)}"
      role="alert"
    >
      <span class="text-lg flex-shrink-0">{getIcon(notification.type)}</span>
      <div class="flex-1 min-w-0">
        <div class="font-medium {getTextColor(notification.type)}">{notification.title}</div>
        {#if notification.message}
          <div class="text-sm mt-1 {getTextColor(notification.type)} opacity-80">{notification.message}</div>
        {/if}
      </div>
      <button
        class="flex-shrink-0 p-1 hover:bg-black/10 rounded {getTextColor(notification.type)}"
        onclick={() => removeNotification(notification.id)}
      >
        ✕
      </button>
    </div>
  {/each}
</div>

<style>
  @keyframes slide-in {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
  .animate-slide-in {
    animation: slide-in 0.3s ease-out;
  }
</style>
