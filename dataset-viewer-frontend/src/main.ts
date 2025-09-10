import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import i18n from './i18n'
import App from './App.vue'

// Styles
import './assets/styles/main.css'

// Create app
const app = createApp(App)

// Install plugins
app.use(createPinia())
app.use(router)
app.use(i18n)

// Global error handler
app.config.errorHandler = (err, vm, info) => {
  console.error('Vue error:', err, info)
}

// Mount app
app.mount('#app')