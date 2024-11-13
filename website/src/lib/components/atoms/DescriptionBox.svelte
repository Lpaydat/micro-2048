<script>
  export let description = "Default game mode description";
</script>

<style>
  .custom-shadow {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2); /* Subtle shadow for depth */
  }
  .custom-transition {
    transition: transform 0.2s;
  }
</style>

<div class="description-box w-full bg-surface-600 text-white p-5 rounded-lg font-sans text-lg text-center custom-shadow custom-transition transform scale-110 border-4 border-surface-800">
  {description}
</div> 