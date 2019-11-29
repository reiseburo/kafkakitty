<template>
  <div class="message">
    <div class="metadata">
      <code>{{msg.offset}}</code>
      <strong>{{msg.topic}}</strong>
      ({{msg.partition}})
    </div>

    <json-viewer
      class="json"
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

<style>
.message {
  display: contents;
  text-align: left;
}

.metadata {
  grid-column: 1;
  text-align: right;
  font-size: 10px;
  padding-top: 5px;
  padding-right: 20px;
}

.json {
  grid-column: 2 / 10;
}
</style>
