<template>
  <div class="kitty">
    <h1>Welcome to Kafkakitty 😺</h1>

    <ul class="messages">
      <li is="Message" v-for="msg in messages" :key="msg.offset" :msg="msg"></li>
    </ul>
  </div>

</template>

<script>
import Message from './Message'

export default {
  name: 'Kitty',
  components: {
    Message
  },
  computed: {
    messages () {
      return this.$store.state.messages
    }
  }
}
</script>

<style>
.messages {
  display: grid;
  grid-gap: 3px;
}
</style>
