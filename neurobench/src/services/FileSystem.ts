import { readDir, readTextFile, writeTextFile, exists, BaseDirectory, mkdir } from '@tauri-apps/plugin-fs';
import { open, save } from '@tauri-apps/plugin-dialog';

// Define the FileEntry interface based on Tauri's FileEntry but simplified for our needs
export interface FileSystemEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: FileSystemEntry[];
}

export class FileSystem {
  
  /**
   * Open a directory selection dialog and return the selected path
   */
  static async openFolder(): Promise<string | null> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Open Project Folder'
      });
      return selected as string | null;
    } catch (error) {
      console.error('Error opening folder:', error);
      return null;
    }
  }

  /**
   * Open a save file dialog and return the selected path
   */
  static async saveFile(options?: { title?: string; defaultPath?: string; filters?: any[] }): Promise<string | null> {
    try {
      const selected = await save({
        title: options?.title || 'Save File',
        defaultPath: options?.defaultPath,
        filters: options?.filters,
      });
      return selected as string | null;
    } catch (error) {
      console.error('Error opening save dialog:', error);
      return null;
    }
  }

  /**
   * Read the contents of a directory directly
   */
  static async readDirectory(path: string): Promise<FileSystemEntry[]> {
    try {
      const entries = await readDir(path);
      
      // Sort: Directories first, then files. Alphabetical order within groups.
      return entries
        .map(entry => ({
            name: entry.name,
            path: `${path}/${entry.name}`, // Construct absolute-like path (simplified)
            isDirectory: !!entry.isDirectory,
            // Children can be lazy loaded or mapped if recursive is needed, 
            // but readDir is shallow by default unless recursive option is used (Tauri v2 specific)
            // For this impl, we assuming shallow read and will recurse in UI or separate calls
        }))
        .sort((a, b) => {
          if (a.isDirectory === b.isDirectory) {
            return a.name.localeCompare(b.name);
          }
          return a.isDirectory ? -1 : 1;
        });
    } catch (error) {
      console.error(`Error reading directory ${path}:`, error);
      return [];
    }
  }

  /**
   * Recursively read a directory to build a tree
   * Note: Be careful with large directories!
   */
  static async readDirectoryRecursive(path: string): Promise<FileSystemEntry[]> {
    const entries = await this.readDirectory(path);
    
    for (const entry of entries) {
      if (entry.isDirectory) {
        entry.children = await this.readDirectoryRecursive(entry.path);
      }
    }
    
    return entries;
  }

  /**
   * Read text file content
   */
  static async readFile(path: string): Promise<string> {
    try {
      return await readTextFile(path);
    } catch (error) {
      console.error(`Error reading file ${path}:`, error);
      throw error;
    }
  }

  /**
   * Write text content to a file
   */
  static async writeFile(path: string, content: string): Promise<void> {
    try {
      await writeTextFile(path, content);
    } catch (error) {
      console.error(`Error writing file ${path}:`, error);
      throw error;
    }
  }

  /**
   * Create a new directory
   */
  static async createDirectory(path: string): Promise<void> {
      try {
          await mkdir(path, { recursive: true });
      } catch (error) {
          console.error(`Error creating directory ${path}:`, error);
          throw error;
      }
  }

  /**
   * Check if file or directory exists
   */
  static async exists(path: string): Promise<boolean> {
      try {
          return await exists(path);
      } catch (error) {
          return false;
      }
  }
}
