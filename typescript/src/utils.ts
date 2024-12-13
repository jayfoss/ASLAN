type StackItem ={
  source: any;
  target: any;
  isArray: boolean;
}

export function deepCopy<T>(source: T): T {
  if (source === null || typeof source !== 'object') {
    return source;
  }

  const stack: StackItem[] = [];
  const isArray = Array.isArray(source);
  const result = isArray ? [] : {};

  stack.push({
    source,
    target: result,
    isArray
  });

  while (stack.length > 0) {
    const { source: currentSource, target: currentTarget, isArray: isCurrentArray } = stack.pop()!;

    const keys = isCurrentArray 
      ? Object.keys(currentSource).map(Number) 
      : Object.keys(currentSource);

    for (const key of keys) {
      const value = currentSource[key];

      if (value === null || typeof value !== 'object') {
        currentTarget[key] = value;
        continue;
      }

      const isValueArray = Array.isArray(value);
      const newTarget = isValueArray ? [] : {};
      currentTarget[key] = newTarget;

      stack.push({
        source: value,
        target: newTarget,
        isArray: isValueArray
      });
    }
  }

  return result as T;
}

export function generateRandomIdempotencyKey() {
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}