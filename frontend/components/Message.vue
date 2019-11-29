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

    <div class="body">
      <pre>
        <code>
          {{msg.payload}}
        </code>
      </pre>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Message',
  props: [
    'msg'
  ]
}
</script>
