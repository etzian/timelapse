// Age screen. Québec: a person under 14 needs the consent of the parent or
// guardian to have their personal information collected (Act respecting the
// protection of personal information in the private sector).
//
// Rules for the screen itself: ask neutrally (a birth date, never "are you over
// 13?"), say why, keep nothing beyond the result, and do not let a refusal be
// undone by re-entering a different date in the same session (`lockedOut`).

/**
 * @param {{ birthDate: string | Date, now?: Date, minimumAge: number, parentalConsentBelow?: number }} o
 * @returns {{ status: 'allowed' | 'parental-consent' | 'blocked', age: number }}
 */
export function evaluateAge({ birthDate, now = new Date(), minimumAge, parentalConsentBelow = 14 }) {
  const b = birthDate instanceof Date ? birthDate : new Date(`${birthDate}T00:00:00Z`);
  if (Number.isNaN(b.getTime()) || b.getTime() > now.getTime()) throw new RangeError('invalid birth date');
  let age = now.getUTCFullYear() - b.getUTCFullYear();
  const beforeBirthday = now.getUTCMonth() < b.getUTCMonth() || (now.getUTCMonth() === b.getUTCMonth() && now.getUTCDate() < b.getUTCDate());
  if (beforeBirthday) age -= 1;
  if (age < minimumAge) return { status: 'blocked', age };
  if (age < parentalConsentBelow) return { status: 'parental-consent', age };
  return { status: 'allowed', age };
}

/** Remembers a refusal for the session so the date cannot simply be retyped. */
export function createAgeGate({ minimumAge, parentalConsentBelow = 14, now = () => new Date() }) {
  let blocked = false;
  return {
    lockedOut: () => blocked,
    check(birthDate) {
      if (blocked) return { status: 'blocked', age: NaN };
      const r = evaluateAge({ birthDate, now: now(), minimumAge, parentalConsentBelow });
      if (r.status === 'blocked') blocked = true;
      return r;
    }
  };
}
