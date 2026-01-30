import { writable } from 'svelte/store';

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

export const notifications = writable<Notification[]>([]);

let notificationId = 0;

export function addNotification(notification: Omit<Notification, 'id'>): string {
  const id = `notification-${++notificationId}`;
  const duration = notification.duration ?? 5000;

  notifications.update(list => [...list, { ...notification, id }]);

  if (duration > 0) {
    setTimeout(() => removeNotification(id), duration);
  }

  return id;
}

export function removeNotification(id: string) {
  notifications.update(list => list.filter(n => n.id !== id));
}

export function clearNotifications() {
  notifications.set([]);
}

// Convenience functions
export function notifySuccess(title: string, message?: string) {
  return addNotification({ type: 'success', title, message });
}

export function notifyError(title: string, message?: string) {
  return addNotification({ type: 'error', title, message, duration: 10000 });
}

export function notifyWarning(title: string, message?: string) {
  return addNotification({ type: 'warning', title, message });
}

export function notifyInfo(title: string, message?: string) {
  return addNotification({ type: 'info', title, message });
}
