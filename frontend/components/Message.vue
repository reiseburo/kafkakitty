<template>
  <div class="message">
    <span class="timestamp"></span>

    <div class="body">
      <pre>
        <code>
          {{msg}}
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
