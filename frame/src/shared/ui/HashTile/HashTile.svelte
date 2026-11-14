<script lang="ts">
  /** H.A.S.H's app icon itself, its own squircle and shadow included, the
   *  same in either theme. */
  let { size = 16 }: { size?: number } = $props();
</script>

<img
  class="hash-tile"
  src="/hash-icon@1x.png"
  srcset="/hash-icon@1x.png 128w, /hash-icon@2x.png 256w"
  sizes={`${size}px`}
  width={size}
  height={size}
  alt=""
  aria-hidden="true"
  draggable="false"
/>

<style>
  .hash-tile {
    display: inline-block;
    flex: none;
    object-fit: contain;
  }
</style>
