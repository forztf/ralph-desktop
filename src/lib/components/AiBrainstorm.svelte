<script lang="ts">
  import type { ProjectState, CliType } from '$lib/types';
  import * as api from '$lib/services/tauri';
  import type { ConversationMessage, AiBrainstormResponse, QuestionOption } from '$lib/services/tauri';
  import { config } from '$lib/stores/settings';

  interface Props {
    project: ProjectState;
    onComplete: (project: ProjectState) => void;
    onCancel: () => void;
  }

  let { project, onComplete, onCancel }: Props = $props();

  let conversation = $state<ConversationMessage[]>([]);
  let currentQuestion = $state<AiBrainstormResponse | null>(null);
  let selectedOptions = $state<Set<string>>(new Set());
  let customInput = $state('');
  let showOtherInput = $state(false);
  let isLoading = $state(false);
  let generatedPrompt = $state<string | null>(null);
  let selectedCli = $state<CliType>($config.defaultCli);
  let maxIterations = $state($config.defaultMaxIterations);

  // Start with initial question
  $effect(() => {
    if (conversation.length === 0 && !currentQuestion) {
      askInitialQuestion();
    }
  });

  async function askInitialQuestion() {
    isLoading = true;
    try {
      // First question: what do you want to do?
      currentQuestion = {
        question: '你想做什么？',
        description: '请简单描述一下你的任务',
        options: [],
        multiSelect: false,
        allowOther: false,
        isComplete: false
      };
    } finally {
      isLoading = false;
    }
  }

  async function submitAnswer(answer: string) {
    if (isLoading) return;

    // Add to conversation
    conversation = [...conversation, { role: 'user', content: answer }];

    // Reset state
    selectedOptions = new Set();
    customInput = '';
    showOtherInput = false;
    isLoading = true;

    try {
      const response = await api.aiBrainstormChat(project.id, conversation);

      if (response.isComplete && response.generatedPrompt) {
        generatedPrompt = response.generatedPrompt;
        currentQuestion = null;
      } else {
        // Add AI response to conversation for context
        conversation = [...conversation, { role: 'assistant', content: response.question }];
        currentQuestion = response;
      }
    } catch (error) {
      console.error('Failed to get AI response:', error);
      // Fallback question
      currentQuestion = {
        question: '请继续描述你的需求',
        description: '出现了一些问题，请重试',
        options: [],
        multiSelect: false,
        allowOther: false,
        isComplete: false
      };
    } finally {
      isLoading = false;
    }
  }

  function handleOptionClick(option: QuestionOption) {
    if (currentQuestion?.multiSelect) {
      const newSet = new Set(selectedOptions);
      if (newSet.has(option.value)) {
        newSet.delete(option.value);
      } else {
        newSet.add(option.value);
      }
      selectedOptions = newSet;
    } else {
      // Single select - submit immediately
      submitAnswer(option.label);
    }
  }

  function handleOtherClick() {
    if (currentQuestion?.multiSelect) {
      showOtherInput = !showOtherInput;
    } else {
      showOtherInput = true;
    }
  }

  function handleSubmitMultiple() {
    const answers = Array.from(selectedOptions);
    if (showOtherInput && customInput.trim()) {
      answers.push(customInput.trim());
    }
    if (answers.length > 0) {
      submitAnswer(answers.join(', '));
    }
  }

  function handleSubmitText() {
    if (customInput.trim()) {
      submitAnswer(customInput.trim());
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      if (currentQuestion?.options.length === 0 || showOtherInput) {
        handleSubmitText();
      }
    }
    if (event.key === 'Escape') {
      onCancel();
    }
  }

  async function handleComplete() {
    if (!generatedPrompt) return;

    isLoading = true;
    try {
      const updatedProject = await api.completeAiBrainstorm(
        project.id,
        generatedPrompt,
        selectedCli,
        maxIterations
      );
      onComplete(updatedProject);
    } catch (error) {
      console.error('Failed to complete brainstorm:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="flex-1 flex flex-col h-full bg-gray-900">
  <!-- Header with tabs style -->
  <div class="px-6 pt-6 pb-4 border-b border-gray-700">
    <div class="flex items-center gap-4 text-sm text-gray-400">
      {#each conversation.filter(m => m.role === 'user').slice(-3) as msg, i}
        <span class="truncate max-w-32" class:text-blue-400={i === conversation.filter(m => m.role === 'user').length - 1}>
          {msg.content.slice(0, 20)}{msg.content.length > 20 ? '...' : ''}
        </span>
        {#if i < Math.min(2, conversation.filter(m => m.role === 'user').length - 1)}
          <span class="text-gray-600">→</span>
        {/if}
      {/each}
    </div>
    {#if conversation.length > 0}
      <div class="mt-2 h-0.5 bg-gray-700 rounded">
        <div class="h-full bg-blue-500 rounded transition-all" style="width: {Math.min(100, conversation.length * 15)}%"></div>
      </div>
    {/if}
  </div>

  <!-- Main content -->
  <div class="flex-1 overflow-y-auto px-6 py-8">
    {#if isLoading}
      <div class="flex items-center gap-3 text-gray-400">
        <div class="animate-spin h-5 w-5 border-2 border-gray-600 border-t-blue-500 rounded-full"></div>
        <span>AI 正在思考...</span>
      </div>
    {:else if generatedPrompt}
      <!-- Completion state -->
      <div class="space-y-6">
        <div>
          <h2 class="text-xl font-semibold text-white mb-2">需求收集完成 ✓</h2>
          <p class="text-gray-400">以下是根据你的需求生成的任务描述</p>
        </div>

        <div class="bg-gray-800 rounded-lg p-4 max-h-64 overflow-y-auto">
          <pre class="text-sm text-gray-300 whitespace-pre-wrap font-mono">{generatedPrompt}</pre>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm text-gray-400 mb-2">选择 CLI</label>
            <select
              class="w-full p-3 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
              bind:value={selectedCli}
            >
              <option value="claude">Claude Code</option>
              <option value="codex">Codex</option>
            </select>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-2">最大迭代次数</label>
            <input
              type="number"
              class="w-full p-3 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
              bind:value={maxIterations}
              min="1"
              max="100"
            />
          </div>
        </div>
      </div>
    {:else if currentQuestion}
      <!-- Question display -->
      <div class="space-y-6">
        <div>
          <h2 class="text-xl font-semibold text-white mb-2">{currentQuestion.question}</h2>
          {#if currentQuestion.description}
            <p class="text-gray-400">{currentQuestion.description}</p>
          {/if}
        </div>

        <!-- Options as cards -->
        {#if currentQuestion.options.length > 0}
          <div class="space-y-3">
            {#each currentQuestion.options as option}
              <button
                class="w-full text-left p-4 rounded-lg border transition-all
                  {selectedOptions.has(option.value)
                    ? 'bg-blue-900/30 border-blue-500 text-white'
                    : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600 hover:bg-gray-750'}"
                onclick={() => handleOptionClick(option)}
              >
                <div class="flex items-start gap-3">
                  <div class="mt-0.5 w-5 h-5 rounded border-2 flex items-center justify-center flex-shrink-0
                    {selectedOptions.has(option.value) ? 'border-blue-500 bg-blue-500' : 'border-gray-600'}">
                    {#if selectedOptions.has(option.value)}
                      <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                      </svg>
                    {/if}
                  </div>
                  <div>
                    <div class="font-medium">{option.label}</div>
                    {#if option.description}
                      <div class="text-sm text-gray-500 mt-1">{option.description}</div>
                    {/if}
                  </div>
                </div>
              </button>
            {/each}

            <!-- Other option -->
            {#if currentQuestion.allowOther}
              <button
                class="w-full text-left p-4 rounded-lg border transition-all
                  {showOtherInput
                    ? 'bg-blue-900/30 border-blue-500 text-white'
                    : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600'}"
                onclick={handleOtherClick}
              >
                <div class="flex items-start gap-3">
                  <div class="mt-0.5 w-5 h-5 rounded border-2 flex items-center justify-center flex-shrink-0
                    {showOtherInput ? 'border-blue-500 bg-blue-500' : 'border-gray-600'}">
                    {#if showOtherInput}
                      <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                      </svg>
                    {/if}
                  </div>
                  <div class="font-medium">Other</div>
                </div>
              </button>
            {/if}
          </div>

          <!-- Custom input for "Other" -->
          {#if showOtherInput}
            <div class="mt-4">
              <textarea
                class="w-full p-4 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 resize-none focus:border-blue-500 focus:outline-none"
                rows="2"
                placeholder="请输入..."
                bind:value={customInput}
                onkeydown={handleKeydown}
              ></textarea>
            </div>
          {/if}
        {:else}
          <!-- Text input only -->
          <div>
            <textarea
              class="w-full p-4 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 resize-none focus:border-blue-500 focus:outline-none"
              rows="4"
              placeholder="请输入..."
              bind:value={customInput}
              onkeydown={handleKeydown}
            ></textarea>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between">
    <div class="text-sm text-gray-500">
      {#if !generatedPrompt}
        <kbd class="px-2 py-1 bg-gray-800 rounded text-xs">Esc</kbd> 取消
      {/if}
    </div>

    <div class="flex gap-3">
      {#if generatedPrompt}
        <button
          class="px-4 py-2 text-gray-400 hover:text-white"
          onclick={onCancel}
        >
          取消
        </button>
        <button
          class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium disabled:opacity-50"
          onclick={handleComplete}
          disabled={isLoading}
        >
          开始执行 →
        </button>
      {:else if currentQuestion}
        {#if currentQuestion.multiSelect && (selectedOptions.size > 0 || (showOtherInput && customInput.trim()))}
          <button
            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium"
            onclick={handleSubmitMultiple}
          >
            <span class="text-gray-400 mr-2">{selectedOptions.size + (showOtherInput && customInput.trim() ? 1 : 0)}</span>
            Submit answers
          </button>
        {:else if currentQuestion.options.length === 0}
          <button
            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium disabled:opacity-50"
            onclick={handleSubmitText}
            disabled={!customInput.trim()}
          >
            继续 →
          </button>
        {:else if showOtherInput && !currentQuestion.multiSelect}
          <button
            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium disabled:opacity-50"
            onclick={handleSubmitText}
            disabled={!customInput.trim()}
          >
            继续 →
          </button>
        {/if}
      {/if}
    </div>
  </div>
</div>
