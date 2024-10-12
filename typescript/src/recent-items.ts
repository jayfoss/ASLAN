export class RecentItems<T> {
  private items: T[];
  private maxSize: number;

  constructor(maxSize: number = 5) {
    this.items = [];
    this.maxSize = maxSize;
  }

  add(item: T): void {
    this.items.unshift(item);

    if (this.items.length > this.maxSize) {
      this.items.pop();
    }
  }

  getItems(): T[] {
    return [...this.items];
  }

  getMostRecent(): T | undefined {
    return this.items[0];
  }

  getNthMostRecent(n: number): T | undefined {
    if (n < 1 || n > this.items.length) {
      return undefined;
    }
    return this.items[n - 1];
  }

  getMostRecentNotIn(excludedSet: Set<T>): T | undefined {
    for (const item of this.items) {
      if (!excludedSet.has(item)) {
        return item;
      }
    }
    return undefined;
  }

  getNthMostRecentNotIn(n: number, excludedSet: Set<T>): T | undefined {
    if (n < 1) {
      throw new Error("n must be a positive integer");
    }

    let count = 0;
    for (const item of this.items) {
      if (!excludedSet.has(item)) {
        count++;
        if (count === n) {
          return item;
        }
      }
    }
    
    return undefined;
  }
}