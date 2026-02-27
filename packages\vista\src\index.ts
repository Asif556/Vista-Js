// Export all modules from subdirectories
export * from './router';
export * from './components';
export * from './auth';
export * from './types';
export * from './dev-error';

// Client-side features
export { 
  useRouter, 
  usePathname, 
  useSearchParams, 
  useParams, 
  useSelectedLayoutSegment, 
  useSelectedLayoutSegments 
} from './client/navigation';
export { default as dynamic } from './client/dynamic';
export { 
  default as Script, 
  default as Head, 
  generateMetadataHead, 
  type HeadMetadata 
} from './client/head';

// Font and metadata exports
export * from './client/font';
export * from './metadata';
export type { Metadata } from './metadata/types';

// RSC (React Server Components) exports
export { 
  hydrateClientComponents, 
  initializeHydration, 
  ClientIsland, 
  Client 
} from './client';

// Build system exports (for advanced usage)
export type { 
  ClientManifest, 
  ClientComponentEntry, 
  ServerManifest, 
  ServerComponentEntry, 
  RouteEntry, 
  RSCPayload, 
  ClientReference 
} from './build/rsc';