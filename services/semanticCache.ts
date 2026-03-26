/**
 * Semantic Cache - Cache AI responses to reduce costs and latency
 * Uses content hashing to identify similar prompts
 */

interface CacheEntry {
  hash: string;
  prompt: string;
  response: string;
  model: string;
  taskType: string;
  timestamp: number;
  ttl: number; // Time to live in ms
  hits: number;
}

interface CacheStats {
  totalEntries: number;
  totalHits: number;
  totalMisses: number;
  hitRate: number;
  sizeMB: number;
}

class SemanticCache {
  private cache: Map<string, CacheEntry> = new Map();
  private totalHits: number = 0;
  private totalMisses: number = 0;
  
  // Default TTL: 24 hours
  private defaultTTL: number = 24 * 60 * 60 * 1000;
  
  // Max cache size: 100MB
  private maxSizeBytes: number = 100 * 1024 * 1024;
  
  constructor() {
    // Load from localStorage if available
    this.loadFromStorage();
    
    // Periodic cleanup
    setInterval(() => this.cleanup(), 60000); // Every minute
  }
  
  /**
   * Generate hash for a prompt + config combination
   */
  private generateHash(prompt: string, taskType: string): string {
    // Simple hash function for browser compatibility
    let hash = 0;
    const str = `${taskType}::${prompt}`;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return `cache_${Math.abs(hash).toString(16)}`;
  }
  
  /**
   * Normalize prompt for better cache hits
   */
  private normalizePrompt(prompt: string): string {
    return prompt
      .trim()
      .replace(/\s+/g, ' ') // Normalize whitespace
      .toLowerCase();
  }
  
  /**
   * Get cached response if available
   */
  get(prompt: string, taskType: string): string | null {
    const normalizedPrompt = this.normalizePrompt(prompt);
    const hash = this.generateHash(normalizedPrompt, taskType);
    
    const entry = this.cache.get(hash);
    if (!entry) {
      this.totalMisses++;
      return null;
    }
    
    // Check if expired
    if (Date.now() > entry.timestamp + entry.ttl) {
      this.cache.delete(hash);
      this.totalMisses++;
      return null;
    }
    
    // Update hit count
    entry.hits++;
    this.totalHits++;
    
    return entry.response;
  }
  
  /**
   * Store response in cache
   */
  set(prompt: string, taskType: string, response: string, model: string, ttl?: number): void {
    const normalizedPrompt = this.normalizePrompt(prompt);
    const hash = this.generateHash(normalizedPrompt, taskType);
    
    const entry: CacheEntry = {
      hash,
      prompt: normalizedPrompt.substring(0, 500), // Store truncated for debugging
      response,
      model,
      taskType,
      timestamp: Date.now(),
      ttl: ttl || this.defaultTTL,
      hits: 0
    };
    
    this.cache.set(hash, entry);
    this.saveToStorage();
    
    // Check size limit
    this.enforceMaxSize();
  }
  
  /**
   * Invalidate cache entry
   */
  invalidate(prompt: string, taskType: string): void {
    const normalizedPrompt = this.normalizePrompt(prompt);
    const hash = this.generateHash(normalizedPrompt, taskType);
    this.cache.delete(hash);
    this.saveToStorage();
  }
  
  /**
   * Clear all cache
   */
  clear(): void {
    this.cache.clear();
    this.totalHits = 0;
    this.totalMisses = 0;
    this.saveToStorage();
  }
  
  /**
   * Get cache statistics
   */
  getStats(): CacheStats {
    const totalRequests = this.totalHits + this.totalMisses;
    
    let totalSize = 0;
    this.cache.forEach(entry => {
      totalSize += entry.response.length + entry.prompt.length;
    });
    
    return {
      totalEntries: this.cache.size,
      totalHits: this.totalHits,
      totalMisses: this.totalMisses,
      hitRate: totalRequests > 0 ? this.totalHits / totalRequests : 0,
      sizeMB: totalSize / (1024 * 1024)
    };
  }
  
  /**
   * Remove expired entries
   */
  private cleanup(): void {
    const now = Date.now();
    const toDelete: string[] = [];
    
    this.cache.forEach((entry, hash) => {
      if (now > entry.timestamp + entry.ttl) {
        toDelete.push(hash);
      }
    });
    
    toDelete.forEach(hash => this.cache.delete(hash));
    
    if (toDelete.length > 0) {
      this.saveToStorage();
    }
  }
  
  /**
   * Enforce maximum cache size by removing least used entries
   */
  private enforceMaxSize(): void {
    const stats = this.getStats();
    
    if (stats.sizeMB * 1024 * 1024 > this.maxSizeBytes) {
      // Sort by hits (ascending) and timestamp (oldest first)
      const entries = Array.from(this.cache.entries())
        .sort((a, b) => {
          if (a[1].hits !== b[1].hits) return a[1].hits - b[1].hits;
          return a[1].timestamp - b[1].timestamp;
        });
      
      // Remove entries until under limit
      let currentSize = stats.sizeMB * 1024 * 1024;
      for (const [hash, entry] of entries) {
        if (currentSize <= this.maxSizeBytes * 0.8) break; // Leave 20% buffer
        
        this.cache.delete(hash);
        currentSize -= (entry.response.length + entry.prompt.length);
      }
      
      this.saveToStorage();
    }
  }
  
  /**
   * Save cache to localStorage
   */
  private saveToStorage(): void {
    try {
      if (typeof localStorage === 'undefined') return;
      
      const data = {
        entries: Array.from(this.cache.entries()),
        totalHits: this.totalHits,
        totalMisses: this.totalMisses
      };
      
      localStorage.setItem('neurostate_ai_cache', JSON.stringify(data));
    } catch (e) {
      // Storage might be full or unavailable
      console.warn('Failed to save cache:', e);
    }
  }
  
  /**
   * Load cache from localStorage
   */
  private loadFromStorage(): void {
    try {
      if (typeof localStorage === 'undefined') return;
      
      const data = localStorage.getItem('neurostate_ai_cache');
      if (!data) return;
      
      const parsed = JSON.parse(data);
      this.cache = new Map(parsed.entries || []);
      this.totalHits = parsed.totalHits || 0;
      this.totalMisses = parsed.totalMisses || 0;
      
      // Cleanup expired on load
      this.cleanup();
    } catch (e) {
      console.warn('Failed to load cache:', e);
    }
  }
}

// Export singleton instance
export const semanticCache = new SemanticCache();
export { SemanticCache };
export type { CacheEntry, CacheStats };
