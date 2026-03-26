/**
 * Neurostate File System Utility
 * Safe file operations with sandboxing and validation
 */

import * as fs from 'fs/promises';
import * as path from 'path';

export interface FileSystemConfig {
  sandboxRoot: string;
  allowedExtensions?: string[];
  maxFileSize?: number; // in bytes
}

export interface FileResult {
  success: boolean;
  content?: string;
  path?: string;
  error?: string;
  metadata?: {
    size: number;
    created: Date;
    modified: Date;
  };
}

export class FileSystem {
  private config: FileSystemConfig;

  constructor(config: FileSystemConfig) {
    this.config = config;
  }

  /**
   * Validate that a path is within the sandbox
   */
  private validatePath(requestedPath: string): string {
    const resolved = path.resolve(requestedPath);
    const sandboxResolved = path.resolve(this.config.sandboxRoot);

    if (!resolved.startsWith(sandboxResolved)) {
      throw new Error(`Path ${requestedPath} is outside sandbox ${sandboxResolved}`);
    }

    // Check extension if allowedExtensions is specified
    if (this.config.allowedExtensions && this.config.allowedExtensions.length > 0) {
      const ext = path.extname(resolved);
      if (!this.config.allowedExtensions.includes(ext)) {
        throw new Error(`File extension ${ext} is not allowed`);
      }
    }

    return resolved;
  }

  /**
   * Read a file with safety checks
   */
  async readFile(filePath: string): Promise<FileResult> {
    try {
      const safePath = this.validatePath(filePath);
      
      const stats = await fs.stat(safePath);
      
      // Check file size
      if (this.config.maxFileSize && stats.size > this.config.maxFileSize) {
        return {
          success: false,
          error: `File size ${stats.size} exceeds maximum ${this.config.maxFileSize}`
        };
      }

      const content = await fs.readFile(safePath, 'utf-8');

      return {
        success: true,
        content,
        path: safePath,
        metadata: {
          size: stats.size,
          created: stats.birthtime,
          modified: stats.mtime
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error reading file'
      };
    }
  }

  /**
   * Write a file with safety checks
   */
  async writeFile(filePath: string, content: string): Promise<FileResult> {
    try {
      const safePath = this.validatePath(filePath);
      
      // Check content size
      if (this.config.maxFileSize && Buffer.byteLength(content) > this.config.maxFileSize) {
        return {
          success: false,
          error: `Content size exceeds maximum ${this.config.maxFileSize} bytes`
        };
      }

      // Ensure directory exists
      const dir = path.dirname(safePath);
      await fs.mkdir(dir, { recursive: true });

      await fs.writeFile(safePath, content, 'utf-8');

      const stats = await fs.stat(safePath);

      return {
        success: true,
        path: safePath,
        metadata: {
          size: stats.size,
          created: stats.birthtime,
          modified: stats.mtime
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error writing file'
      };
    }
  }

  /**
   * List files in a directory
   */
  async listFiles(dirPath: string, recursive: boolean = false): Promise<FileResult & { files?: string[] }> {
    try {
      const safePath = this.validatePath(dirPath);
      
      const files: string[] = [];
      
      const walk = async (currentPath: string) => {
        const entries = await fs.readdir(currentPath, { withFileTypes: true });
        
        for (const entry of entries) {
          const fullPath = path.join(currentPath, entry.name);
          
          if (entry.isDirectory()) {
            if (recursive) {
              await walk(fullPath);
            }
          } else {
            // Check extension filter
            if (!this.config.allowedExtensions || 
                this.config.allowedExtensions.includes(path.extname(fullPath))) {
              files.push(fullPath);
            }
          }
        }
      };

      await walk(safePath);

      return {
        success: true,
        path: safePath,
        files
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error listing files'
      };
    }
  }

  /**
   * Delete a file
   */
  async deleteFile(filePath: string): Promise<FileResult> {
    try {
      const safePath = this.validatePath(filePath);
      await fs.unlink(safePath);

      return {
        success: true,
        path: safePath
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error deleting file'
      };
    }
  }

  /**
   * Check if file exists
   */
  async exists(filePath: string): Promise<boolean> {
    try {
      const safePath = this.validatePath(filePath);
      await fs.access(safePath);
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * Create a default file system instance for agents
 */
export function createAgentFileSystem(sandboxRoot: string): FileSystem {
  return new FileSystem({
    sandboxRoot,
    allowedExtensions: ['.ts', '.js', '.json', '.md', '.txt', '.yaml', '.yml', '.toml', '.c', '.h', '.cpp'],
    maxFileSize: 10 * 1024 * 1024 // 10MB
  });
}
