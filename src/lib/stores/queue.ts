import { writable, derived, get } from 'svelte/store';
import type { ProjectMeta } from '../types';
import * as api from '$lib/services/tauri';

// Queue of projects waiting to run
export const projectQueue = writable<string[]>([]);

// Currently running projects
export const runningProjects = writable<Set<string>>(new Set());

// Max concurrent projects (synced from config)
export const maxConcurrent = writable<number>(3);

// Derived: queue status
export const queueStatus = derived(
  [projectQueue, runningProjects, maxConcurrent],
  ([$queue, $running, $max]) => ({
    queueLength: $queue.length,
    runningCount: $running.size,
    canStartMore: $running.size < $max,
    availableSlots: $max - $running.size
  })
);

// Add project to queue
export function enqueueProject(projectId: string) {
  projectQueue.update(queue => {
    if (!queue.includes(projectId)) {
      return [...queue, projectId];
    }
    return queue;
  });
  processQueue();
}

// Remove from queue
export function dequeueProject(projectId: string) {
  projectQueue.update(queue => queue.filter(id => id !== projectId));
}

// Mark project as running
export function markRunning(projectId: string) {
  runningProjects.update(set => {
    const newSet = new Set(set);
    newSet.add(projectId);
    return newSet;
  });
}

// Mark project as stopped
export function markStopped(projectId: string) {
  runningProjects.update(set => {
    const newSet = new Set(set);
    newSet.delete(projectId);
    return newSet;
  });
  // Process queue when a slot opens up
  processQueue();
}

// Process queue - start next project if slots available
async function processQueue() {
  const status = get(queueStatus);
  const queue = get(projectQueue);

  if (status.canStartMore && queue.length > 0) {
    const nextProjectId = queue[0];
    dequeueProject(nextProjectId);
    markRunning(nextProjectId);

    try {
      await api.startLoop(nextProjectId);
    } catch (error) {
      console.error('Failed to start queued project:', error);
      markStopped(nextProjectId);
    }
  }
}

// Check if project is in queue
export function isInQueue(projectId: string): boolean {
  return get(projectQueue).includes(projectId);
}

// Check if project is running
export function isRunning(projectId: string): boolean {
  return get(runningProjects).has(projectId);
}

// Get queue position (1-indexed, 0 if not in queue)
export function getQueuePosition(projectId: string): number {
  const queue = get(projectQueue);
  const index = queue.indexOf(projectId);
  return index === -1 ? 0 : index + 1;
}
