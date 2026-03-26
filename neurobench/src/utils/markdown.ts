/**
 * Markdown Renderer using highlight.js directly for code blocks
 * Simple parser for bold, italic, code blocks, and lists.
 */
import hljs from 'highlight.js/lib/core';
import javascript from 'highlight.js/lib/languages/javascript';
import typescript from 'highlight.js/lib/languages/typescript';
import json from 'highlight.js/lib/languages/json';
import rust from 'highlight.js/lib/languages/rust';
import html from 'highlight.js/lib/languages/xml';
import css from 'highlight.js/lib/languages/css';
import 'highlight.js/styles/atom-one-dark.css';

// Register languages
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('json', json);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('html', html);
hljs.registerLanguage('css', css);

export function renderMarkdown(text: string): string {
  if (!text) return "";

  let html = text;

  // Escape HTML (basic)
  html = html
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  // Code Blocks (```lang ... ```)
  html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, (_match, lang, code) => {
    let highlightedCode = code;
    try {
        // Decode HTML entities for highlight.js 
        const decodedCode = code.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
        if (lang && hljs.getLanguage(lang)) {
            highlightedCode = hljs.highlight(decodedCode, { language: lang }).value;
        } else {
            highlightedCode = hljs.highlightAuto(decodedCode).value;
        }
    } catch (e) {
        console.warn("Highlight error", e);
    }
    return `<pre><code class="hljs ${lang || ''}">${highlightedCode}</code></pre>`;
  });

  // Inline Code (`...`)
  html = html.replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>');

  // Bold (**...**)
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');

  // Italic (*...*)
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>');

  // Headers (### ...)
  html = html.replace(/^### (.*$)/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.*$)/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.*$)/gm, '<h1>$1</h1>');

  // Lists (- ...)
  html = html.replace(/^\- (.*$)/gm, '<li>$1</li>');
  // Wrap li in ul (simple heuristic: if we have multiple li, wrap them)
  // For simplicity in this regex parser, we might leave them as is or try to wrap blocks.
  // A better approach for lists in regex is tough, so we'll accept <li> for now, which browser renders okayish or we style it.
  
  // Newlines to <br> (but not inside pre/code)
  // This is tricky with regex. For now, we rely on the fact that code blocks are already replaced.
  // We can treat remaining newlines as <br>? 
  // Better: CSS `white-space: pre-wrap` handles newlines better than <br> for regular text.
  // So we won't aggressively replace \n unless it's double \n for paragraph.
  
  html = html.replace(/\n\n/g, '<br/><br/>'); // Paragraphs

  return html;
}
