
export const BRAND_BORDER = 'm15 10c-2.7614 0-5 2.2386-5 5v98c0 2.761 2.2386 5 5 5h178c2.761 0 5-2.239 5-5v-98c0-2.7614-2.239-5-5-5zm-15 5c0-8.28427 6.71573-15 15-15h178c8.284 0 15 6.71573 15 15v98c0 8.284-6.716 15-15 15h-178c-8.28427 0-15-6.716-15-15z';
export const BRAND_LETTER = 'M30 30h70v16h-27v52h-16v-52h-27z';
export const BRAND_ARROW = 'm155 98-30-33h20v-35h20v35h20z';
export const BRAND_VIEWBOX = '0 0 208 128';
export const BRAND_COLOR = '#a0522d';

// Pre-encoded SVG for use as a data URI favicon
const svg = `<svg fill='none' viewBox='${BRAND_VIEWBOX}' xmlns='http://www.w3.org/2000/svg'>`
  + `<path clip-rule='evenodd' d='${BRAND_BORDER}' fill-rule='evenodd' fill='${BRAND_COLOR}'/>`
  + `<path d='${BRAND_LETTER}' fill='${BRAND_COLOR}'/>`
  + `<path d='${BRAND_ARROW}' fill='${BRAND_COLOR}'/>`
  + '</svg>';

export const BRAND_FAVICON_URI = `data:image/svg+xml,${encodeURIComponent(svg)}`;
