import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { Quasar, Dialog, Notify, QBadge, QBtn, QTooltip } from 'quasar'
import '@quasar/extras/material-icons/material-icons.css'
import 'quasar/src/css/index.sass'
import './styles/main.scss'
import App from './App.vue'

createApp(App)
  .use(createPinia())
  .use(Quasar, {
    components: { QBadge, QBtn, QTooltip },
    plugins: { Dialog, Notify },
    config: {
      brand: {
        primary: '#168cff',
        secondary: '#3ab8ff',
        dark: '#080d14',
      },
      notify: {
        position: 'bottom-right',
        timeout: 2400,
      },
    },
  })
  .mount('#app')
