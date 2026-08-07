export interface ResolvedRef {
  url: string;
  name: string;
}

export function extractRef (value: unknown): ResolvedRef | undefined {
  if (typeof value !== 'object' || value === null) return undefined;
  const ref_ = (value as Record<string, unknown>).$ref;

  if (typeof ref_ !== 'object' || ref_ === null) return undefined;
  const {
    url, name,
  } = ref_ as Record<string, unknown>;

  if (typeof url !== 'string' || typeof name !== 'string') return undefined;

  return {
    url,
    name,
  };
}
