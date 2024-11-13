<script>
  export let src = '';
  export let alt = 'Image';
</script>

<style>
  img {
    width: 100%;
    height: auto;
    max-width: 800px; /* Adjust this value to control the maximum size */
    display: block;
    margin: 0 auto;
  }
</style>

<img {src} {alt} />
