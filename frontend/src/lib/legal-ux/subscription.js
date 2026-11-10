// Subscription disclosure (Consumer Protection Act, distance contracts; Apple
// 3.1.2). Everything must be visible before the user pays, in the same place
// as the buy button: price with taxes noted, billing period, renewal, trial
// conversion, how to cancel.

export const REQUIRED = ['price', 'period', 'renewal', 'cancel'];

/**
 * @param {{ price?: string, period?: string, renewal?: string, trial?: string, cancel?: string, taxes?: string }} d
 * @returns {string[]} the required fields that are empty
 */
export function missingDisclosures(d) {
  return REQUIRED.filter((k) => !d?.[k] || !String(d[k]).trim());
}
