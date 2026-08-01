import {
  defineComponent, h,
} from 'vue';
import {
  useRoute,
} from '../router';

// Renders the current page's compiled .td content
export const Content = defineComponent({
  name: 'TypedownContent',
  setup () {
    const route = useRoute();

    return () => route.contentSfc ? h(route.contentSfc) : undefined;
  },
});
