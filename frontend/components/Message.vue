<template>
  <div class="message">

    <span class="timestamp">{{timestamp}}</span>

    <div class="body">
      <pre>
        <code>
          {{body}}
        </code>
      </pre>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Message',
  data () {
    return {
    }
  }
}
</script>
