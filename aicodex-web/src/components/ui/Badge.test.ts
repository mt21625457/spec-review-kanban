/**
 * Badge.vue component tests
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import Badge from './Badge.vue'

describe('Badge', () => {
  it('renders slot content', () => {
    const wrapper = mount(Badge, {
      slots: {
        default: 'Status',
      },
    })

    expect(wrapper.text()).toBe('Status')
  })

  it('has badge class', () => {
    const wrapper = mount(Badge)

    expect(wrapper.classes()).toContain('badge')
  })

  it('applies default variant classes when no variant', () => {
    const wrapper = mount(Badge)

    expect(wrapper.classes()).toContain('bg-bg-secondary')
    expect(wrapper.classes()).toContain('text-text-normal')
  })

  it('applies default variant classes explicitly', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'default' },
    })

    expect(wrapper.classes()).toContain('bg-bg-secondary')
    expect(wrapper.classes()).toContain('text-text-normal')
  })

  it('applies pending variant class', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'pending' },
    })

    expect(wrapper.classes()).toContain('badge-pending')
  })

  it('applies running variant class', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'running' },
    })

    expect(wrapper.classes()).toContain('badge-running')
  })

  it('applies completed variant class', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'completed' },
    })

    expect(wrapper.classes()).toContain('badge-completed')
  })

  it('applies failed variant class', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'failed' },
    })

    expect(wrapper.classes()).toContain('badge-failed')
  })

  it('applies cancelled variant class', () => {
    const wrapper = mount(Badge, {
      props: { variant: 'cancelled' },
    })

    expect(wrapper.classes()).toContain('badge-cancelled')
  })

  it('renders as a span element', () => {
    const wrapper = mount(Badge)

    expect(wrapper.element.tagName).toBe('SPAN')
  })
})
