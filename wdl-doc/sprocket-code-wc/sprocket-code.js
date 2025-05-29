import { createHighlighterCore } from 'shiki/core';
import catppuccinMocha from '@shikijs/themes/catppuccin-mocha';
import rust from '@shikijs/langs/rust';
import wdlGrammar from './wdl.tmGrammar.json';
import { createOnigurumaEngine } from 'shiki/engine/oniguruma';
import wasm from 'shiki/wasm';

class SprocketCode extends HTMLElement {
  static get observedAttributes() {
    return ['language'];
  }

  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.highlighter = null;
    this.language = 'rust';
    this.code = '';
  }

  async connectedCallback() {
    console.log('sprocket-code: Connected callback');
    try {
      // Initialize highlighter with explicit language and theme
      this.highlighter = await createHighlighterCore({
        themes: [catppuccinMocha],
        langs: [rust, wdlGrammar],
        engine: createOnigurumaEngine(wasm)
      });

      // Get initial content
      this.code = this.textContent.trim();
      this.textContent = ''; // Clear the element's content
      await this.highlight(this.code);
    } catch (error) {
      console.error('Failed to initialize highlighter:', error);
    }
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (name === 'language' && newValue && oldValue !== newValue) {
      this.language = newValue;
      if (this.highlighter && this.code) {
        this.highlight(this.code);
      }
    }
  }

  async highlight(code) {
    if (!this.highlighter || !code) return;
    
    try {
      const html = this.highlighter.codeToHtml(code, {
        lang: this.language,
        theme: 'catppuccin-mocha'
      });

      this.shadowRoot.innerHTML = `
        <style>
          :host {
            display: block;
          }
          .code-block {
            margin: 1em 0;
            border-radius: 6px;
            overflow: hidden;
          }
          .code-block pre {
            margin: 0;
            padding: 1em;
            overflow-x: auto;
          }
          /* Override Shiki's background to match our theme */
          .code-block :global(pre.shiki) {
            background: var(--shiki-bg, #2e3440) !important;
          }
        </style>
        <div class="code-block">
          ${html}
        </div>
      `;
    } catch (error) {
      console.error('Failed to highlight code:', error);
      this.shadowRoot.innerHTML = `
        <pre><code>${code}</code></pre>
      `;
    }
  }
}

customElements.define('sprocket-code', SprocketCode);