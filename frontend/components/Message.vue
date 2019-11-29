<template>
  <div class="message">
    <span class="timestamp"></span>
    <span class="offset">
      {{msg.offset}}
    </span>
    <span class="partition">
      {{msg.partition}}
    </span>
    <span class="topic">
      {{msg.topic}}
    </span>

    <json-viewer
      :value="msg.payload"
      :expand-depth=5
      copyable
      boxed
      sort>
    </json-viewer>
  </div>
</template>

<script>
import JsonViewer from 'vue-json-viewer'

export default {
  name: 'Message',
  props: [
    'msg'
  ],
  components: [
    JsonViewer
  ]
}
</script>
