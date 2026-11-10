import { invoke } from './invoke';

export interface AppInfo {
  version: string;
}
export interface Greeting {
  name: string;
}

export const appInfo = () => invoke<AppInfo>('app_info');
export const greet = (name: string) => invoke<Greeting>('greet', { name });
