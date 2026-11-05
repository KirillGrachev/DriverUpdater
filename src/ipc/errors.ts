export interface CommandErrorInfo {
    code: string | null;
    message: string;
}

/**
 * Normalizes both serialized Rust commands and legacy string errors.
 */
export function commandErrorInfo(error: unknown): CommandErrorInfo {
    if (error && typeof error === 'object') {
        const candidate = error as { code?: unknown; message?: unknown };
        return {
            code: typeof candidate.code === 'string' ? candidate.code : null,
            message: typeof candidate.message === 'string' ? candidate.message : String(error),
        };
    }
    const message = typeof error === 'string' ? error : String(error);
    return {
        code: message.startsWith('E_RESTORE_POINT')
            ? 'restorePoint'
            : message === 'E_PACKAGE_CANCELLED'
              ? 'packageCancelled'
              : null,
        message,
    };
}
