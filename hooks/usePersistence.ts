import { useState, useEffect, useRef } from 'react';
import { FSMProject } from '../types';

const STORAGE_KEY = 'neurostate_db_v1';
const ACTIVE_PROJECT_KEY = 'neurostate_active_id';

export function usePersistence(initialProjects: FSMProject[], initialActiveId: string) {
  const [projects, setProjects] = useState<FSMProject[]>(initialProjects);
  const [activeProjectId, setActiveProjectId] = useState<string>(initialActiveId);
  const [isLoaded, setIsLoaded] = useState(false);

  // Load from Storage on Mount
  useEffect(() => {
    try {
      const storedProjects = localStorage.getItem(STORAGE_KEY);
      const storedActiveId = localStorage.getItem(ACTIVE_PROJECT_KEY);

      if (storedProjects) {
        const parsedProjects = JSON.parse(storedProjects);
        // Basic validation
        if (Array.isArray(parsedProjects) && parsedProjects.length > 0) {
          setProjects(parsedProjects);
        }
      }

      if (storedActiveId) {
        setActiveProjectId(storedActiveId);
      }
    } catch (e) {
      console.error("Failed to load persistence data:", e);
    } finally {
      setIsLoaded(true);
    }
  }, []);

  // Save to Storage on Change (Debounced)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!isLoaded) return;

    if (timeoutRef.current) clearTimeout(timeoutRef.current);

    timeoutRef.current = setTimeout(() => {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(projects));
        localStorage.setItem(ACTIVE_PROJECT_KEY, activeProjectId);
        console.log('Saved to LocalStorage');
      } catch (e) {
        console.error("Failed to save persistence data:", e);
      }
    }, 1000); // 1-second debounce

    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [projects, activeProjectId, isLoaded]);

  return { 
    projects, 
    setProjects, 
    activeProjectId, 
    setActiveProjectId,
    isLoaded 
  };
}