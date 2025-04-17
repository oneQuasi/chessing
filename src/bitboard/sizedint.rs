use num::{BigUint, Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Num, NumCast, PrimInt, Saturating, ToPrimitive, Unsigned};
use num_traits::{One, Zero};
use std::{cmp::Ordering, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub}};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct SizedInt<const T: usize>(pub [ u64; T ]);

impl<const T: usize> PartialOrd for SizedInt<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const T: usize> Ord for SizedInt<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..T {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}

impl<const T: usize> SizedInt<T> {
    fn zero() -> Self {
        Self([0u64; T])
    }

    fn one() -> Self {
        let mut arr = [0u64; T];
        arr[T - 1] = 1;
        Self(arr)
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }

    fn shl1(&self) -> Self {
        let mut result = [0u64; T];
        let mut carry = 0u64;

        for i in (0..T).rev() {
            result[i] = (self.0[i] << 1) | carry;
            carry = (self.0[i] >> 63) & 1;
        }

        Self(result)
    }

    fn shr1(&self) -> Self {
        let mut result = [0u64; T];
        let mut carry = 0u64;

        for i in 0..T {
            result[i] = (self.0[i] >> 1) | (carry << 63);
            carry = self.0[i] & 1;
        }

        Self(result)
    }

    pub fn to_biguint(&self) -> BigUint {
        let mut result = BigUint::zero();
        for &limb in &self.0 {
            result <<= 64;
            result |= BigUint::from(limb);
        }
        result
    }

    pub fn from_biguint(n: BigUint) -> Self {
        let mut limbs = [0u64; T];
        let mut n = n;

        for i in (0..T).rev() {
            if n.is_zero() {
                break;
            }
            limbs[i] = (&n & BigUint::from(u64::MAX)).to_u64().unwrap_or(0);
            n >>= 64;
        }

        Self(limbs)
    }
}

impl<const T: usize> Add for SizedInt<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [0u64; T];
        let mut carry = 0u64;

        for i in (0..T).rev() {
            let (sum1, overflow1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum2, overflow2) = sum1.overflowing_add(carry);
            res[i] = sum2;
            carry = (overflow1 as u64) + (overflow2 as u64);
        }

        Self(res)
    }
}

impl<const T: usize> Zero for SizedInt<T> {
    fn zero() -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> bool {
        self.is_zero()
    }
}

impl<const T: usize> Mul for SizedInt<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = [0u64; T];

        for i in (0..T).rev() {
            for j in (0..T).rev() {
                // Calculate target index from the right
                let result_idx = i + j + 1 - T;

                if result_idx < T {
                    let a = self.0[i];
                    let b = rhs.0[j];
                    let (low, carry) = a.overflowing_mul(b);

                    // Add to result
                    let (sum1, overflow1) = result[result_idx].overflowing_add(low);
                    let mut carry = carry as u64 + overflow1 as u64;

                    result[result_idx] = sum1;

                    // Propagate carry
                    let mut k = result_idx;
                    while carry > 0 && k > 0 {
                        k -= 1;
                        let (sum2, overflow2) = result[k].overflowing_add(carry);
                        result[k] = sum2;
                        carry = overflow2 as u64;
                    }
                }
            }
        }

        Self(result)
    }
}

impl<const T: usize> One for SizedInt<T> {
    fn one() -> Self {
        Self::one()
    }

    fn is_one(&self) -> bool {
        self.0.iter().enumerate().all(|(i, &x)| {
            if i == T - 1 {
                x == 1
            } else {
                x == 0
            }
        })
    }
}

impl<const T: usize> Sub for SizedInt<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = [0u64; T];
        let mut borrow = 0u64;

        for i in (0..T).rev() {
            let (mut diff, mut overflow) = self.0[i].overflowing_sub(rhs.0[i] + borrow);
            if overflow {
                diff = diff.wrapping_sub(1);
                borrow = 1;
            } else {
                borrow = 0;
            }
            result[i] = diff;
        }

        if borrow != 0 {
            panic!("Subtraction underflow in SizedInt");
        }

        Self(result)
    }
}

impl<const T: usize> Div for SizedInt<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            panic!("division by zero");
        }

        if self < rhs {
            return SizedInt::zero();
        }

        let mut quotient = SizedInt::zero();
        let mut remainder = self;

        let mut denom = rhs;
        let mut shift = 0;

        // Align denom with highest bit of numerator
        while denom <= remainder {
            denom = denom.shl1();
            shift += 1;
        }

        for _ in 0..shift {
            denom = denom.shr1();

            quotient = quotient.shl1();
            if remainder >= denom {
                remainder = remainder - denom;
                quotient.0[T - 1] |= 1;
            }
        }

        quotient
    }
}

impl<const T: usize> BitAnd for SizedInt<T> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut result = [0u64; T];
        for i in 0..T {
            result[i] = self.0[i] & rhs.0[i];
        }
        Self(result)
    }
}

impl<const T: usize> BitOr for SizedInt<T> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = [0u64; T];
        for i in 0..T {
            result[i] = self.0[i] | rhs.0[i];
        }
        Self(result)
    }
}

impl<const T: usize> BitXor for SizedInt<T> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut result = [0u64; T];
        for i in 0..T {
            result[i] = self.0[i] ^ rhs.0[i];
        }
        Self(result)
    }
}

impl<const T: usize> Not for SizedInt<T> {
    type Output = Self;
    fn not(self) -> Self::Output {
        let mut result = [0u64; T];
        for i in 0..T {
            result[i] = !self.0[i];
        }
        Self(result)
    }
}

impl<const T: usize> Shl<u32> for SizedInt<T> {
    type Output = Self;

    fn shl(self, mut shift: u32) -> Self::Output {
        let mut result = self.0;

        while shift >= 64 {
            for i in 0..T - 1 {
                result[i] = result[i + 1];
            }
            result[T - 1] = 0;
            shift -= 64;
        }

        if shift > 0 {
            let mut carry = 0u64;
            for i in (0..T).rev() {
                let new_carry = result[i] >> (64 - shift);
                result[i] = (result[i] << shift) | carry;
                carry = new_carry;
            }
        }

        Self(result)
    }
}

impl<const T: usize> Shr<u32> for SizedInt<T> {
    type Output = Self;

    fn shr(self, mut shift: u32) -> Self::Output {
        let mut result = self.0;

        while shift >= 64 {
            for i in (1..T).rev() {
                result[i] = result[i - 1];
            }
            result[0] = 0;
            shift -= 64;
        }

        if shift > 0 {
            let mut carry = 0u64;
            for i in 0..T {
                let new_carry = result[i] << (64 - shift);
                result[i] = (result[i] >> shift) | carry;
                carry = new_carry;
            }
        }

        Self(result)
    }
}

impl<const T: usize> PrimInt for SizedInt<T> {
    fn count_ones(self) -> u32 {
        self.0.iter().map(|&x| x.count_ones()).sum()
    }

    fn count_zeros(self) -> u32 {
        self.0.iter().map(|&x| x.count_zeros()).sum()
    }

    fn leading_zeros(self) -> u32 {
        for i in 0..T {
            if self.0[i] != 0 {
                return (i as u32) * 64 + self.0[i].leading_zeros();
            }
        }
        (T as u32) * 64
    }

    fn trailing_zeros(self) -> u32 {
        for i in (0..T).rev() {
            if self.0[i] != 0 {
                return ((T - 1 - i) as u32) * 64 + self.0[i].trailing_zeros();
            }
        }
        (T as u32) * 64
    }

    fn rotate_left(self, mut n: u32) -> Self {
        n %= (T as u32) * 64;
        let total_bits = T * 64;
        let mut result = self;
        result = (result << n) | (result >> (total_bits as u32 - n));
        result
    }

    fn rotate_right(self, mut n: u32) -> Self {
        n %= (T as u32) * 64;
        let total_bits = T * 64;
        let mut result = self;
        result = (result >> n) | (result << (total_bits as u32 - n));
        result
    }

    fn pow(self, mut exp: u32) -> Self {
        let mut base = self;
        let mut result = Self::one();

        while exp > 0 {
            if exp % 2 == 1 {
                result = result * base;
            }
            base = base * base;
            exp /= 2;
        }

        result
    }

    fn swap_bytes(self) -> Self {
        let mut result = [0u64; T];
        for i in 0..T {
            result[i] = self.0[i].swap_bytes();
        }
        result.reverse();
        Self(result)
    }

    fn to_be(self) -> Self {
        #[cfg(target_endian = "little")]
        {
            self.swap_bytes()
        }

        #[cfg(target_endian = "big")]
        {
            *self
        }
    }

    fn to_le(self) -> Self {
        #[cfg(target_endian = "big")]
        {
            self.swap_bytes()
        }

        #[cfg(target_endian = "little")]
        {
            self
        }
    }

    fn from_be(x: Self) -> Self {
        x.to_be()
    }

    fn from_le(x: Self) -> Self {
        x.to_le()
    }

    fn signed_shl(self, rhs: u32) -> Self {
        self << rhs
    }

    fn signed_shr(self, rhs: u32) -> Self {
        self >> rhs
    }

    fn unsigned_shl(self, rhs: u32) -> Self {
        self << rhs
    }

    fn unsigned_shr(self, rhs: u32) -> Self {
        self >> rhs
    }
}
impl<const T: usize> Unsigned for SizedInt<T> {}

impl<const T: usize> Saturating for SizedInt<T> {
    fn saturating_add(self, rhs: Self) -> Self {
        let mut res = [0u64; T];
        let mut carry = 0u64;

        for i in (0..T).rev() {
            let (sum1, overflow1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum2, overflow2) = sum1.overflowing_add(carry);
            res[i] = sum2;
            carry = (overflow1 as u64) + (overflow2 as u64);
        }

        if carry > 0 {
            // Overflow occurred → return max value
            Self([u64::MAX; T])
        } else {
            Self(res)
        }
    }

    fn saturating_sub(self, rhs: Self) -> Self {
        if self < rhs {
            Self::zero()
        } else {
            self - rhs
        }
    }
}

impl<const T: usize> CheckedDiv for SizedInt<T> {
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(*self / *rhs)
        }
    }
}

impl<const T: usize> Bounded for SizedInt<T> {
    fn min_value() -> Self {
        Self::zero()
    }

    fn max_value() -> Self {
        Self([u64::MAX; T])
    }
}

impl<const T: usize> CheckedAdd for SizedInt<T> {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let mut res = [0u64; T];
        let mut carry = 0u64;

        for i in (0..T).rev() {
            let (sum1, overflow1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum2, overflow2) = sum1.overflowing_add(carry);
            res[i] = sum2;
            carry = (overflow1 as u64) + (overflow2 as u64);
        }

        if carry > 0 {
            None
        } else {
            Some(Self(res))
        }
    }
}

impl<const T: usize> CheckedSub for SizedInt<T> {
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        let mut res = [0u64; T];
        let mut borrow = 0u64;

        for i in (0..T).rev() {
            let (diff, overflow1) = self.0[i].overflowing_sub(rhs.0[i] + borrow);
            if overflow1 {
                borrow = 1;
            } else {
                borrow = 0;
            }
            res[i] = diff;
        }

        if borrow > 0 {
            None
        } else {
            Some(Self(res))
        }
    }
}

impl<const T: usize> CheckedMul for SizedInt<T> {
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        // Try doing multiplication manually and check for overflow
        let mut result = Self::zero();

        for i in (0..T).rev() {
            for j in (0..T).rev() {
                let result_idx = i + j + 1 - T;
                if result_idx < T {
                    let a = self.0[i];
                    let b = rhs.0[j];
                    let (low, overflow_mul) = a.overflowing_mul(b);

                    let (sum, overflow_add1) = result.0[result_idx].overflowing_add(low);
                    let mut carry = overflow_mul as u64 + overflow_add1 as u64;

                    result.0[result_idx] = sum;

                    let mut k = result_idx;
                    while carry > 0 && k > 0 {
                        k -= 1;
                        let (sum2, overflow_add2) = result.0[k].overflowing_add(carry);
                        result.0[k] = sum2;
                        carry = overflow_add2 as u64;

                        if carry > 0 && k == 0 {
                            // can't carry out of bounds
                            return None;
                        }
                    }
                } else {
                    return None; // overflowed limb count
                }
            }
        }

        Some(result)
    }
}

impl<const T: usize> Shl<usize> for SizedInt<T> {
    type Output = Self;

    fn shl(self, shift: usize) -> Self::Output {
        self.shl(shift as u32)
    }
}

impl<const T: usize> Shr<usize> for SizedInt<T> {
    type Output = Self;

    fn shr(self, shift: usize) -> Self::Output {
        self.shr(shift as u32)
    }
}

impl<const T: usize> NumCast for SizedInt<T> {
    fn from<TSrc: ToPrimitive>(n: TSrc) -> Option<Self> {
        let mut arr = [0u64; T];
        if let Some(val) = n.to_u64() {
            arr[T - 1] = val;
            Some(Self(arr))
        } else {
            None
        }
    }
}

impl<const T: usize> From<u64> for SizedInt<T> {
    fn from(n: u64) -> Self {
        let mut arr = [0u64; T];
        arr[T - 1] = n;
        Self(arr)
    }
}

impl<const T: usize> ToPrimitive for SizedInt<T> {
    fn to_i64(&self) -> Option<i64> {
        self.to_u64().map(|x| x as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        if T == 0 {
            return Some(0);
        }
        if T == 1 || self.0[0..T - 1].iter().all(|&x| x == 0) {
            Some(self.0[T - 1] as u64)
        } else {
            None
        }
    }

    fn to_f32(&self) -> Option<f32> {
        self.to_f64().map(|f| f as f32)
    }

    fn to_f64(&self) -> Option<f64> {
        let mut result: f64 = 0.0;
        for (i, &limb) in self.0.iter().enumerate() {
            if limb != 0 {
                result += (limb as f64) * 2f64.powi(((T - 1 - i) * 64) as i32);
            }
        }
        Some(result)
    }

    // Optional: You can implement more if needed (to_i32, to_u32, etc.)
}

impl<const T: usize> Rem for SizedInt<T> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            panic!("attempt to calculate the remainder with a zero divisor");
        }

        let lhs_big = self.to_biguint();
        let rhs_big = rhs.to_biguint();
        let result = lhs_big % rhs_big;

        Self::from_biguint(result)
    }
}


impl<const T: usize> Num for SizedInt<T> {
    type FromStrRadixErr = <BigUint as Num>::FromStrRadixErr;

    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let big = BigUint::from_str_radix(src, radix)?;
        let mut limbs = [0u64; T];
        let mut n = big.clone();

        for i in (0..T).rev() {
            let limb = &n & BigUint::from(u64::MAX);
            limbs[i] = limb.to_u64().unwrap_or(0);
            n >>= 64;
        }

        Ok(Self(limbs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    // A helper to create from u64
    fn from_u64<const T: usize>(x: u64) -> SizedInt<T> {
        let mut data = [0u64; T];
        if T > 0 {
            data[T - 1] = x;
        }
        SizedInt(data)
    }

    #[test]
    fn test_equality() {
        let a = SizedInt([1u64, 2]);
        let b = SizedInt([1u64, 2]);
        let c = SizedInt([0u64, 2]);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_clone_copy() {
        let a = SizedInt([5u64, 10]);
        let b = a;
        let c = a.clone();

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn test_from_u64() {
        let x = from_u64::<2>(42);
        assert_eq!(x, SizedInt([0, 42]));

        let y = from_u64::<1>(999);
        assert_eq!(y, SizedInt([999]));
    }

    #[test]
    fn test_debug_format() {
        let x = SizedInt([1u64, 2]);
        let s = format!("{:?}", x);
        assert!(s.contains("1") && s.contains("2"));
    }

    #[test]
    fn test_addition_simple() {
        let a = from_u64::<2>(1);
        let b = from_u64::<2>(2);
        let c = from_u64::<2>(3);

        assert_eq!(add(a, b), c);
    }

    #[test]
    fn test_addition_carry() {
        let a = SizedInt([1, u64::MAX]);
        let b = from_u64::<2>(1);
        let expected = SizedInt([2, 0]);

        assert_eq!(add(a, b), expected);
    }

    #[test]
    fn test_bitshift_left() {
        let x = from_u64::<2>(1);
        let shifted = shl(x, 64);
        assert_eq!(shifted, SizedInt([1, 0]));
    }

    #[test]
    fn test_bitshift_right() {
        let x = SizedInt([1, 0]);
        let shifted = shr(x, 64);
        assert_eq!(shifted, from_u64::<2>(1));
    }

    #[test]
    fn test_zero() {
        let zero = SizedInt::<3>([0; 3]);
        assert!(is_zero(&zero));
    }

    // Helpers for basic arithmetic, since they're not shown in your type.
    fn add<const T: usize>(a: SizedInt<T>, b: SizedInt<T>) -> SizedInt<T> {
        let mut result = [0u64; T];
        let mut carry = 0u64;

        for i in (0..T).rev() {
            let (sum1, overflow1) = a.0[i].overflowing_add(b.0[i]);
            let (sum2, overflow2) = sum1.overflowing_add(carry);
            result[i] = sum2;
            carry = (overflow1 as u64) + (overflow2 as u64);
        }

        SizedInt(result)
    }

    fn shl<const T: usize>(x: SizedInt<T>, shift: usize) -> SizedInt<T> {
        let bits_per = 64;
        let mut result = [0u64; T];

        for i in 0..T {
            let src_index = i + shift / bits_per;
            if src_index >= T {
                continue;
            }

            let bit_shift = shift % bits_per;
            let lower = x.0[src_index] << bit_shift;
            let upper = if bit_shift > 0 && src_index + 1 < T {
                x.0[src_index + 1] >> (bits_per - bit_shift)
            } else {
                0
            };

            result[i] = lower | upper;
        }

        SizedInt(result)
    }

    fn shr<const T: usize>(x: SizedInt<T>, shift: usize) -> SizedInt<T> {
        let bits_per = 64;
        let mut result = [0u64; T];

        for i in (0..T).rev() {
            let src_index = i as isize - (shift / bits_per) as isize;
            if src_index < 0 || src_index as usize >= T {
                continue;
            }

            let bit_shift = shift % bits_per;
            let upper = x.0[src_index as usize] >> bit_shift;
            let lower = if bit_shift > 0 && src_index > 0 {
                x.0[(src_index - 1) as usize] << (bits_per - bit_shift)
            } else {
                0
            };

            result[i] = upper | lower;
        }

        SizedInt(result)
    }

    fn is_zero<const T: usize>(x: &SizedInt<T>) -> bool {
        x.0.iter().all(|&v| v == 0)
    }
}
