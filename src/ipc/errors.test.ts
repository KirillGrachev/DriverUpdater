import { describe, expect, it } from 'vitest';
import { commandErrorInfo } from './errors';

describe('commandErrorInfo', () => {
    it('reads serialized command errors', () => {
        expect(commandErrorInfo({ code: 'restorePoint', message: 'disabled' })).toEqual({
            code: 'restorePoint',
            message: 'disabled',
        });
    });

    it('classifies legacy restore errors', () => {
        expect(commandErrorInfo('E_RESTORE_POINT: disabled')).toEqual({
            code: 'restorePoint',
            message: 'E_RESTORE_POINT: disabled',
        });
    });

    it('classifies package cancellation', () => {
        expect(commandErrorInfo('E_PACKAGE_CANCELLED').code).toBe('packageCancelled');
    });

    it('keeps unknown errors as internal-free messages', () => {
        expect(commandErrorInfo(new Error('boom'))).toMatchObject({ code: null, message: 'boom' });
    });
});
