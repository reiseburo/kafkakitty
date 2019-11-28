<template>
  <div id="app">
    <Kitty/>
  </div>
</template>

<script>
import Kitty from './components/Kitty'

export default {
  name: 'App',
  components: {
    Kitty
  }
}
</script>

<style>
#app {
  font-family: 'Avenir', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
