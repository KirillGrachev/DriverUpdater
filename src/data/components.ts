export type ComponentGroup = 'runtime' | 'language' | 'tool';

export interface AppComponent {
    id: string;
    name: string;
    vendor: string;
    group: ComponentGroup;
    version: string;
    sizeMb: number;
    installedByDefault: boolean;
}

/**
 * Component catalogue shown by the preview.
 *
 * <p>Installation is simulated locally until a real package backend is wired
 * in; the shape already matches what such a backend would return.</p>
 */
export const COMPONENT_CATALOG: AppComponent[] = [
    {
        id: 'dotnet-framework-4-8',
        name: '.NET Framework',
        vendor: 'Microsoft',
        group: 'runtime',
        version: '4.8.1',
        sizeMb: 112,
        installedByDefault: true,
    },
    {
        id: 'dotnet-desktop-8',
        name: '.NET Desktop Runtime',
        vendor: 'Microsoft',
        group: 'runtime',
        version: '8.0.11',
        sizeMb: 57,
        installedByDefault: false,
    },
    {
        id: 'directx-end-user',
        name: 'DirectX End-User Runtime',
        vendor: 'Microsoft',
        group: 'runtime',
        version: '9.29.1974',
        sizeMb: 96,
        installedByDefault: true,
    },
    {
        id: 'vcredist-2015-2022',
        name: 'Visual C++ Redistributable',
        vendor: 'Microsoft',
        group: 'runtime',
        version: '14.40.33810',
        sizeMb: 25,
        installedByDefault: false,
    },
    {
        id: 'python-3-12',
        name: 'Python',
        vendor: 'Python Software Foundation',
        group: 'language',
        version: '3.12.7',
        sizeMb: 27,
        installedByDefault: false,
    },
    {
        id: 'nodejs-22',
        name: 'Node.js',
        vendor: 'OpenJS Foundation',
        group: 'language',
        version: '22.9.0',
        sizeMb: 30,
        installedByDefault: false,
    },
    {
        id: 'openjdk-21',
        name: 'OpenJDK',
        vendor: 'Eclipse Adoptium',
        group: 'language',
        version: '21.0.4',
        sizeMb: 190,
        installedByDefault: false,
    },
    {
        id: 'go-1-23',
        name: 'Go',
        vendor: 'Google',
        group: 'language',
        version: '1.23.1',
        sizeMb: 65,
        installedByDefault: false,
    },
    {
        id: 'rust-1-81',
        name: 'Rust',
        vendor: 'Rust Foundation',
        group: 'language',
        version: '1.81.0',
        sizeMb: 320,
        installedByDefault: false,
    },
    {
        id: 'php-8-3',
        name: 'PHP',
        vendor: 'The PHP Group',
        group: 'language',
        version: '8.3.11',
        sizeMb: 30,
        installedByDefault: false,
    },
    {
        id: 'git-2-46',
        name: 'Git',
        vendor: 'Git Project',
        group: 'tool',
        version: '2.46.0',
        sizeMb: 60,
        installedByDefault: true,
    },
    {
        id: 'seven-zip-24',
        name: '7-Zip',
        vendor: 'Igor Pavlov',
        group: 'tool',
        version: '24.08',
        sizeMb: 2,
        installedByDefault: false,
    },
    {
        id: 'cmake-3-30',
        name: 'CMake',
        vendor: 'Kitware',
        group: 'tool',
        version: '3.30.3',
        sizeMb: 30,
        installedByDefault: false,
    },
];

const STORAGE_KEY = 'driver-updater.components.v1';

export function loadInstalledComponents(): Set<string> {
    const defaults = COMPONENT_CATALOG.filter((item) => item.installedByDefault).map((item) => item.id);
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return new Set(defaults);
        const saved = JSON.parse(raw) as unknown;
        if (!Array.isArray(saved)) return new Set(defaults);
        return new Set([...defaults, ...saved.filter((id): id is string => typeof id === 'string')]);
    } catch {
        return new Set(defaults);
    }
}

export function saveInstalledComponents(ids: Set<string>): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify([...ids]));
    } catch {
        /** Storage is optional in the preview. */
    }
}
