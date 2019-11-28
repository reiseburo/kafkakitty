// The Vue build version to load with the `import` command
// (runtime-only or standalone) has been set in webpack.base.conf with an alias.
import Vue from 'vue'
import App from './App'

Vue.config.productionTip = false

console.log('Kafkakitty bootstrapping..')

function wsConnect (app) {
  const socket = new WebSocket('ws://localhost:8001')
  socket.onopen = (event) => {
    console.log('Kafkakitty connected! 😺')
  }
  socket.onerror = (err) => {
    console.error('😿 Kafkakitty encountered an error:', err)
    socket.close()
    wsConnect()
  }
  socket.onclose = (event) => {
    console.log('Kafkakitty connection lost, retrying..')
    wsConnect()
  }

  socket.onmessage = (event) => {
    console.log(`Received: ${event.data}`)
    const container = document.getElementById('app')
    const d = document.createElement('div')
    d.className = 'row'
    d.innerHTML = `<pre><code>${event.data}</code></pre>`
    container.insertBefore(d, container.firstChild)
  }
}
/* eslint-disable no-new */
const a = new Vue({
  el: '#app',
  components: { App },
  template: '<App/>'
})

wsConnect(a)
