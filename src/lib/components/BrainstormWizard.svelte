<script lang="ts">
  import type { ProjectState, QuestionTemplate, CliType } from '$lib/types';
  import * as api from '$lib/services/tauri';
  import { config, availableClis } from '$lib/stores/settings';

  interface Props {
    project: ProjectState;
    onComplete: (project: ProjectState) => void;
    onCancel: () => void;
  }

  let { project, onComplete, onCancel }: Props = $props();

  let questions = $state<QuestionTemplate[]>([]);
  let currentIndex = $state(0);
  let answers = $state<Record<string, string | string[]>>({});
  let customInput = $state('');
  let selectedOptions = $state<string[]>([]);
  let loading = $state(true);
  let completing = $state(false);

  // Load questions and existing answers
  $effect(() => {
    loadQuestions();
  });

  async function loadQuestions() {
    loading = true;
    try {
      questions = await api.getBrainstormQuestions();

      // Load existing answers
      if (project.brainstorm?.answers) {
        for (const answer of project.brainstorm.answers) {
          const value = typeof answer.answer === 'string' ? answer.answer : answer.answer;
          answers[answer.questionId] = value as string | string[];
        }
      }
    } catch (error) {
      console.error('Failed to load questions:', error);
    } finally {
      loading = false;
    }
  }

  const currentQuestion = $derived(questions[currentIndex]);
  const progress = $derived(questions.length > 0 ? ((currentIndex + 1) / questions.length) * 100 : 0);
  const isLastQuestion = $derived(currentIndex === questions.length - 1);

  function handleOptionSelect(value: string) {
    if (currentQuestion?.questionType === 'multiple') {
      if (selectedOptions.includes(value)) {
        selectedOptions = selectedOptions.filter(v => v !== value);
      } else {
        selectedOptions = [...selectedOptions, value];
      }
    } else {
      selectedOptions = [value];
    }
  }

  async function handleNext() {
    if (!currentQuestion) return;

    let answer: string | string[];

    if (currentQuestion.questionType === 'text') {
      answer = customInput;
    } else if (selectedOptions.includes('other')) {
      answer = customInput || 'other';
    } else if (currentQuestion.questionType === 'multiple') {
      answer = selectedOptions;
    } else {
      answer = selectedOptions[0] || '';
    }

    // Save answer
    answers[currentQuestion.id] = answer;

    try {
      await api.saveBrainstormAnswer(
        project.id,
        currentQuestion.id,
        currentQuestion.question,
        answer
      );
    } catch (error) {
      console.error('Failed to save answer:', error);
    }

    // Reset for next question
    customInput = '';
    selectedOptions = [];

    if (isLastQuestion) {
      await handleComplete();
    } else {
      currentIndex++;
    }
  }

  function handleBack() {
    if (currentIndex > 0) {
      currentIndex--;
      // Restore previous answer
      const prevQuestion = questions[currentIndex];
      const prevAnswer = answers[prevQuestion.id];
      if (prevAnswer) {
        if (Array.isArray(prevAnswer)) {
          selectedOptions = prevAnswer;
        } else if (prevQuestion.questionType === 'text') {
          customInput = prevAnswer;
        } else {
          selectedOptions = [prevAnswer];
        }
      }
    }
  }

  async function handleComplete() {
    completing = true;
    try {
      const cliType: CliType = $availableClis.find(c => c.available)?.cliType || 'claude';
      const maxIterations = $config.defaultMaxIterations;

      const updatedProject = await api.completeBrainstorm(project.id, cliType, maxIterations);
      onComplete(updatedProject);
    } catch (error) {
      console.error('Failed to complete brainstorm:', error);
    } finally {
      completing = false;
    }
  }

  const canProceed = $derived(() => {
    if (!currentQuestion) return false;
    if (currentQuestion.questionType === 'text') {
      return !currentQuestion.required || customInput.trim().length > 0;
    }
    if (selectedOptions.includes('other')) {
      return customInput.trim().length > 0;
    }
    return selectedOptions.length > 0 || !currentQuestion.required;
  });
</script>

<div class="flex-1 flex flex-col bg-white dark:bg-gray-800 overflow-hidden">
  <!-- Header -->
  <div class="p-4 border-b border-gray-200 dark:border-gray-700">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-white">
        Brainstorm - {project.name}
      </h2>
      <button
        class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
        onclick={onCancel}
      >
        ✕
      </button>
    </div>
    <!-- Progress bar -->
    <div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
      <div
        class="h-full bg-blue-600 transition-all duration-300"
        style="width: {progress}%"
      ></div>
    </div>
    <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
      Step {currentIndex + 1} of {questions.length}
    </div>
  </div>

  <!-- Question Content -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    {:else if currentQuestion}
      <div class="max-w-2xl mx-auto">
        <h3 class="text-xl font-medium text-gray-800 dark:text-white mb-2">
          {currentQuestion.question}
        </h3>
        {#if currentQuestion.description}
          <p class="text-gray-500 dark:text-gray-400 mb-6">
            {currentQuestion.description}
          </p>
        {/if}

        {#if currentQuestion.questionType === 'text'}
          <!-- Text input -->
          <textarea
            class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-800 dark:text-white resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            rows="4"
            placeholder="请输入..."
            bind:value={customInput}
          ></textarea>
        {:else}
          <!-- Options -->
          <div class="space-y-3">
            {#each currentQuestion.options as option (option.value)}
              <button
                class="w-full p-4 text-left border rounded-lg transition-colors
                  {selectedOptions.includes(option.value)
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30'
                    : 'border-gray-300 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-700'}"
                onclick={() => handleOptionSelect(option.value)}
              >
                <div class="flex items-start gap-3">
                  <div class="mt-0.5">
                    {#if currentQuestion.questionType === 'multiple'}
                      <div class="w-5 h-5 border-2 rounded {selectedOptions.includes(option.value) ? 'border-blue-500 bg-blue-500' : 'border-gray-400'}">
                        {#if selectedOptions.includes(option.value)}
                          <span class="text-white text-xs flex items-center justify-center h-full">✓</span>
                        {/if}
                      </div>
                    {:else}
                      <div class="w-5 h-5 border-2 rounded-full {selectedOptions.includes(option.value) ? 'border-blue-500' : 'border-gray-400'}">
                        {#if selectedOptions.includes(option.value)}
                          <div class="w-3 h-3 m-0.5 rounded-full bg-blue-500"></div>
                        {/if}
                      </div>
                    {/if}
                  </div>
                  <div class="flex-1">
                    <div class="font-medium text-gray-800 dark:text-white">{option.label}</div>
                    {#if option.description}
                      <div class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{option.description}</div>
                    {/if}
                  </div>
                </div>
              </button>
            {/each}

            {#if currentQuestion.allowOther}
              <button
                class="w-full p-4 text-left border rounded-lg transition-colors
                  {selectedOptions.includes('other')
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30'
                    : 'border-gray-300 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-700'}"
                onclick={() => handleOptionSelect('other')}
              >
                <div class="flex items-start gap-3">
                  <div class="mt-0.5">
                    <div class="w-5 h-5 border-2 rounded-full {selectedOptions.includes('other') ? 'border-blue-500' : 'border-gray-400'}">
                      {#if selectedOptions.includes('other')}
                        <div class="w-3 h-3 m-0.5 rounded-full bg-blue-500"></div>
                      {/if}
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="font-medium text-gray-800 dark:text-white">其他</div>
                    {#if selectedOptions.includes('other')}
                      <input
                        type="text"
                        class="mt-2 w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-800 dark:text-white"
                        placeholder="请输入..."
                        bind:value={customInput}
                        onclick={(e) => e.stopPropagation()}
                      />
                    {/if}
                  </div>
                </div>
              </button>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-between">
    <button
      class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-white disabled:opacity-50"
      onclick={handleBack}
      disabled={currentIndex === 0}
    >
      ← Back
    </button>
    <button
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
      onclick={handleNext}
      disabled={!canProceed() || completing}
    >
      {#if completing}
        处理中...
      {:else if isLastQuestion}
        完成 →
      {:else}
        Next →
      {/if}
    </button>
  </div>
</div>
