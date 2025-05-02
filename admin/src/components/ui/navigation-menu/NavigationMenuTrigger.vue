<script setup lang="ts">
  import { cn } from '@/lib/utils'
  import { ChevronDown } from 'lucide-vue-next'
  import { NavigationMenuTrigger, type NavigationMenuTriggerProps, useForwardProps } from 'reka-ui'
  import { computed, type HTMLAttributes } from 'vue'
  import { navigationMenuTriggerStyle } from '.'

  type SizeType = 'sm' | 'md' | 'lg'

  const props = defineProps<
    NavigationMenuTriggerProps & {
      class?: HTMLAttributes['class']
      size?: SizeType
    }
  >()

  const delegatedProps = computed(() => {
    const { class: _, size, ...delegated } = props

    return delegated
  })

  const forwardedProps = useForwardProps(delegatedProps)

  const sizeClasses = computed(() => {
    switch (props.size) {
      case 'sm':
        return 'text-sm px-2 h-6 rounded-none'
      case 'lg':
        return 'text-lg py-3 px-6'
      case 'md':
      default:
        return 'text-base py-2 px-4'
    }
  })

  const chevronSize = computed(() => {
    switch (props.size) {
      case 'sm':
        return 'h-2 w-2'
      case 'lg':
        return 'h-4 w-4'
      case 'md':
      default:
        return 'h-3 w-3'
    }
  })
</script>

<template>
  <NavigationMenuTrigger
    v-bind="forwardedProps"
    :class="cn(navigationMenuTriggerStyle(), sizeClasses, 'group', props.class)"
  >
    <slot />
    <ChevronDown
      :class="
        cn(
          'relative top-px ml-1 transition duration-300 group-data-[state=open]:rotate-180',
          chevronSize
        )
      "
      aria-hidden="true"
    />
  </NavigationMenuTrigger>
</template>
