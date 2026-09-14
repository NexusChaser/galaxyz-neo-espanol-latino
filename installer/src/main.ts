import { mount } from 'svelte'
import './estilos/tokens.css'
import './estilos/base.css'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
