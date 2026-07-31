import TdLayout from './TdLayout.vue';
import TdNotFound from './TdNotFound.vue';

export {
  TdLayout, TdNotFound,
};
export {
  useCopyCode,
} from './composables/useCopyCode';

export {
  default as TdOverview,
} from './components/TdOverview.vue';

export default {
  Layout: TdLayout,
  NotFound: TdNotFound,
};
