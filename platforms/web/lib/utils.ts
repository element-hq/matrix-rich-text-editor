/*
Copyright 2024 New Vector Ltd.
Copyright 2023 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/**
 * Finds the operating system of the user
 * @returns {string|null} the operating system, `null` if the operating system is unknown
 */
export function getUserOperatingSystem():
    | 'Windows'
    | 'macOS'
    | 'Linux'
    | 'iOS'
    | 'Android'
    | null {
    const userAgent = navigator.userAgent.toLowerCase();
    if (userAgent.includes('iphone') || userAgent.includes('ipad')) {
        return 'iOS';
    } else if (userAgent.includes('android')) {
        return 'Android';
    } else if (userAgent.includes('win')) {
        return 'Windows';
    } else if (userAgent.includes('mac')) {
        return 'macOS';
    } else if (userAgent.includes('linux')) {
        return 'Linux';
    } else {
        return null;
    }
}
