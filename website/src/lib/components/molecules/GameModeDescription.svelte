<script>
  import DescriptionBox from '../atoms/DescriptionBox.svelte';

  export let imageUrl = '';
  export let description = 'Default game mode description';
</script>

<div class="max-w-sm mx-auto">
  <div class="overflow-hidden rounded-t-lg">
    <img src={imageUrl} alt="" class="w-full" />
  </div>
  <div class="flex flex-col items-center">
    <DescriptionBox description={description} />
  </div>
</div>
