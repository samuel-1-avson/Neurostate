import { Node, Edge } from 'reactflow';
import { FSMProject } from '../types';

export const fileManager = {
  saveProject: (project: FSMProject) => {
    const data = JSON.stringify({ 
      ...project,
      meta: {
        exportedAt: new Date().toISOString(),
        appName: 'NeuroState',
        formatVersion: '1.1'
      }
    }, null, 2);
    
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    // Sanitize filename
    const safeName = project.name.replace(/[^a-z0-9]/gi, '_').toLowerCase();
    a.download = `${safeName}_v${project.version || '1.0'}.json`;
    a.click();
    URL.revokeObjectURL(url);
  },

  loadProject: (file: File): Promise<Partial<FSMProject>> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const json = JSON.parse(e.target?.result as string);
          if (Array.isArray(json.nodes) && Array.isArray(json.edges)) {
            // Validate and return project structure
            resolve({
                nodes: json.nodes,
                edges: json.edges,
                name: json.name || file.name.replace('.json', ''),
                description: json.description || '',
                version: json.version || '1.0',
                chatHistory: json.chatHistory || [],
                updatedAt: json.updatedAt || Date.now()
            });
          } else {
            reject(new Error('Invalid Project File Structure'));
          }
        } catch (err) {
          reject(err);
        }
      };
      reader.readAsText(file);
    });
  },

  downloadCode: (code: string, filename: string) => {
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }
};