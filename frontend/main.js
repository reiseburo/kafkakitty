// The Vue build version to load with the `import` command
// (runtime-only or standalone) has been set in webpack.base.conf with an alias.
import Vue from 'vue'
import Vuex from 'vuex'
import JsonViewer from 'vue-json-viewer'
import App from './App'

Vue.config.productionTip = false

console.log('Kafkakitty bootstrapping..')

function wsConnect (app) {
  const socket = new WebSocket('ws://localhost:8001')
  socket.app = app

  socket.onopen = (event) => {
    console.log('Kafkakitty connected! 😺')
  }
  socket.onerror = (err) => {
    console.error('😿 Kafkakitty encountered an error:', err)
    socket.close()
    wsConnect(socket.app)
  }
  socket.onclose = (event) => {
    console.log('Kafkakitty connection lost, retrying..')
    wsConnect(socket.app)
  }

  socket.onmessage = (event) => {
    const data = JSON.parse(event.data)

    /*
     * If this looks like some JSON data, then we might as well try to parse it
     */
    const jsony = ['{', '[', ']', '}']
    if (jsony.includes(data.payload.charAt(0)) &&
        (jsony.includes(data.payload.charAt(data.payload.length - 1)))) {
      data.payload = JSON.parse(data.payload)
    }

    socket.app.$store.commit('receive', data)
  }
}

Vue.use(JsonViewer)
Vue.use(Vuex)

const store = new Vuex.Store({
  state: {
    messages: []
  },
  mutations: {
    receive (state, message) {
      if (state.messages.length > 128) {
        state.messages.pop()
      }
      state.messages.unshift(message)
    }
  }
})

/* eslint-disable no-new */
const a = new Vue({
  el: '#app',
  store,
  components: { App },
  template: '<App/>'
})

wsConnect(a)
