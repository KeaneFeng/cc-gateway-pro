/**
 * Deep clone utility with structuredClone runtime guard and fallback.
 * structuredClone is available in modern browsers and Node.js 17+,
 * but may not be available in older environments.
 */

export function deepClone<T>(obj: T): T {
  // Try structuredClone first (native, fastest)
  if (typeof structuredClone === "function") {
    try {
      return structuredClone(obj);
    } catch {
      // structuredClone fails on certain types (e.g., functions, DOM nodes)
      // Fall through to manual implementation
    }
  }

  // Fallback: JSON-based deep clone (works for most serializable objects)
  if (obj === null || typeof obj !== "object") {
    return obj;
  }

  if (obj instanceof Date) {
    return new Date(obj.getTime()) as T;
  }

  if (obj instanceof Array) {
    return obj.map((item) => deepClone(item)) as T;
  }

  if (obj instanceof Object) {
    const clonedObj = {} as T;
    for (const key in obj) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        clonedObj[key] = deepClone(obj[key]);
      }
    }
    return clonedObj;
  }

  return obj;
}
