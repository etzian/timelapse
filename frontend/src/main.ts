import '@etzianco/ui/tokens';
import './app.css';

import { mount } from 'svelte';

import App from './App.svelte';
import { loadSettings } from './lib/stores/settings.svelte';

// Settings are applied before the first mount so the window never paints in the
// wrong language, theme or density (GUIDELINES §8.3).
loadSettings();

export default mount(App, { target: document.getElementById('app')! });
