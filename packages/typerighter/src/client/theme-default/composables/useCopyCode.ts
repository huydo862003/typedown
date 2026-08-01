import {
  onMounted, onUnmounted,
} from 'vue';

// Register a global click handler that copies code blocks to the clipboard
export function useCopyCode (): void {
  function handleClick (event: MouseEvent): void {
    const button = (event.target as Element)?.closest('.copy');

    if (!button) return;

    const codeBlock = button.closest('div[class*="language-"]');
    const code = codeBlock?.querySelector('code');

    if (!code) return;

    navigator.clipboard.writeText(code.textContent ?? '').catch(() => {});

    button.classList.add('copied');

    const copiedText = button.getAttribute('data-copied') ?? 'Copied';
    const originalTitle = button.getAttribute('title') ?? '';

    button.setAttribute('title', copiedText);

    setTimeout(() => {
      button.classList.remove('copied');
      button.setAttribute('title', originalTitle);
    }, 2000);
  }

  onMounted(() => {
    document.addEventListener('click', handleClick);
  });

  onUnmounted(() => {
    document.removeEventListener('click', handleClick);
  });
}
