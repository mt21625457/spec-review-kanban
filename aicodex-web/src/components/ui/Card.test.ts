/**
 * Card.vue component tests
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import Card from './Card.vue'

describe('Card', () => {
  it('renders default slot content', () => {
    const wrapper = mount(Card, {
      slots: {
        default: 'Card content',
      },
    })

    expect(wrapper.text()).toContain('Card content')
  })

  it('renders header slot when provided', () => {
    const wrapper = mount(Card, {
      slots: {
        header: 'Card Header',
        default: 'Card body',
      },
    })

    expect(wrapper.find('.card-header').exists()).toBe(true)
    expect(wrapper.text()).toContain('Card Header')
  })

  it('does not render header when slot is not provided', () => {
    const wrapper = mount(Card, {
      slots: {
        default: 'Card body',
      },
    })

    expect(wrapper.find('.card-header').exists()).toBe(false)
  })

  it('renders footer slot when provided', () => {
    const wrapper = mount(Card, {
      slots: {
        default: 'Card body',
        footer: 'Card Footer',
      },
    })

    expect(wrapper.find('.card-footer').exists()).toBe(true)
    expect(wrapper.text()).toContain('Card Footer')
  })

  it('does not render footer when slot is not provided', () => {
    const wrapper = mount(Card, {
      slots: {
        default: 'Card body',
      },
    })

    expect(wrapper.find('.card-footer').exists()).toBe(false)
  })

  it('applies elevated variant classes', () => {
    const wrapper = mount(Card, {
      props: { variant: 'elevated' },
    })

    expect(wrapper.classes()).toContain('shadow-lg')
  })

  it('applies bordered variant classes', () => {
    const wrapper = mount(Card, {
      props: { variant: 'bordered' },
    })

    expect(wrapper.classes()).toContain('border-2')
  })

  it('has card class by default', () => {
    const wrapper = mount(Card)

    expect(wrapper.classes()).toContain('card')
  })

  it('renders all slots together', () => {
    const wrapper = mount(Card, {
      slots: {
        header: 'Header',
        default: 'Body',
        footer: 'Footer',
      },
    })

    expect(wrapper.find('.card-header').text()).toBe('Header')
    expect(wrapper.find('.card-body').text()).toBe('Body')
    expect(wrapper.find('.card-footer').text()).toBe('Footer')
  })
})
