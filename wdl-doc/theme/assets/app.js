console.log('Entered')

class HelloWorldComponent extends HTMLElement {
  connectedCallback() {
    const msg = this.getAttribute('message') || 'Hello World';
    alert(msg);
    this.innerText = msg;
  }
}
customElements.define('hello-world', HelloWorldComponent);
