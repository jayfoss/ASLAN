export default function convert(json: string, aslanPrefix: string): string {
  let jsonObject;
  try {
    jsonObject = JSON.parse(json);
  } catch (error) {
    throw new Error('Invalid JSON');
  }
  const stack: Array<{ obj: any; keys: string[]; index?: number }> = [];
  if(Array.isArray(jsonObject)) {
    stack.push({ obj: jsonObject, keys: [], index: 0 });
  } else {
    stack.push({ obj: jsonObject, keys: Object.keys(jsonObject) });
  }

  let aslanOutput = '';

  while (stack.length > 0) {
    const current = stack[stack.length - 1];
    
    if (current.keys.length === 0 && !('index' in current)) {
      stack.pop();
      continue;
    }

    if ('index' in current) {
      // Processing an array
      if (current.index! >= current.obj.length) {
        stack.pop();
        continue;
      }
      
      const value = current.obj[current.index!];
      current.index!++;

      if (Array.isArray(value)) {
        aslanOutput += `[${aslanPrefix}a]`;
        stack.push({ obj: value, keys: [], index: 0 });
      } else if (typeof value === 'object' && value !== null) {
        aslanOutput += `[${aslanPrefix}o]`;
        stack.push({ obj: value, keys: Object.keys(value) });
      } else {
        aslanOutput += `[${aslanPrefix}d]${value}`;
      }
    } else {
      // Processing an object
      const key = current.keys.shift()!;
      const value = current.obj[key];

      if (Array.isArray(value)) {
        aslanOutput += `[${aslanPrefix}d_${key}][${aslanPrefix}a]`;
        stack.push({ obj: value, keys: [], index: 0 });
      } else if (typeof value === 'object' && value !== null) {
        aslanOutput += `[${aslanPrefix}d_${key}][${aslanPrefix}o]`;
        stack.push({ obj: value, keys: Object.keys(value) });
      } else {
        aslanOutput += `[${aslanPrefix}d_${key}]${value}`;
      }
    }
  }
  return aslanOutput;
}
