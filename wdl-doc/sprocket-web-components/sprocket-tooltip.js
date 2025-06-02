class SprocketTooltip extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
  }

  _getStyles() {
    return `
      :host { display: inline-block; position: relative; }
      .tooltip {
        position: absolute;
        background: #333;
        color: white;
        padding: 6px 12px;
        border-radius: 4px;
        font-size: 14px;
        white-space: nowrap;
        pointer-events: none;
        opacity: 0;
        transition: opacity 0.2s;
      }
      .tooltip.visible {
        opacity: 1;
      }
      /* Position variants */
      .tooltip[data-position='top'] {
        bottom: 100%;
        left: 50%;
        transform: translateX(-50%) translateY(-8px);
      }
      .tooltip[data-position='bottom'] {
        top: 100%;
        left: 50%;
        transform: translateX(-50%) translateY(8px);
      }
      .tooltip[data-position='left'] {
        right: 100%;
        top: 50%;
        transform: translateX(-8px) translateY(-50%);
      }
      .tooltip[data-position='right'] {
        left: 100%;
        top: 50%;
        transform: translateX(8px) translateY(-50%);
      }
    `;
  }

  connectedCallback() {
    const position = this.getAttribute('position') || 'top';
    const tooltip = this.getAttribute('tooltip') || '';

    this.shadowRoot.innerHTML = `
      <style>${this._getStyles()}</style>
      <div class="tooltip" data-position="${position}">${tooltip}</div>
      <slot></slot>
    `;

    const tooltipElement = this.shadowRoot.querySelector('.tooltip');

    this.addEventListener('mouseenter', () => {
      debugger;
      tooltipElement.classList.add('visible');
    });

    this.addEventListener('mouseleave', () => {
      tooltipElement.classList.remove('visible');
    });
  }
}

customElements.define('sprocket-tooltip', SprocketTooltip);