import { mountSuspended } from '@nuxt/test-utils/runtime';
import { describe, expect, it } from 'vitest';

import Index from '../pages/index.vue';

describe('index page', () => {
  it('renders the welcome heading', async () => {
    const page = await mountSuspended(Index);

    expect(page.get('h1').text()).toBe('Welcome');
  });
});
