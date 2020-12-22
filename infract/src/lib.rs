use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::num::NonZeroU64;
use std::ops;

fn euclid<T>(mut a: T, mut b: T) -> T
where
    T: Copy + Default + std::cmp::PartialEq + ops::Rem<Output = T>,
{
    while b != T::default() {
        let h = a % b;
        a = b;
        b = h;
    }
    a
}

#[derive(Clone, Debug)]
pub struct Fraction {
    primfacts: BTreeMap<u8, i32>,
    numerat: i64,
    denominat: NonZeroU64,
}

impl Default for Fraction {
    fn default() -> Self {
        Self {
            primfacts: [2, 3, 5, 7, 11, 13, 17, 19]
                .iter()
                .copied()
                .map(|prim| (prim, 0))
                .collect(),
            numerat: 0,
            denominat: 1.try_into().unwrap(),
        }
    }
}

impl Fraction {
    fn normalize_inner(numerat: &mut i128, denominat: &mut i128) {
        let rft = euclid(*numerat, *denominat);
        *numerat /= rft;
        *denominat /= rft;
    }

    fn normalize(&mut self) {
        if self.numerat == 0 {
            *self = Fraction::default();
        } else {
            let (mut new_numerat, mut new_denominat): (i128, i128) =
                (self.numerat.into(), self.denominat.get().into());
            Self::normalize_inner(&mut new_numerat, &mut new_denominat);
            self.numerat = new_numerat.try_into().unwrap();
            self.denominat = u64::try_from(new_denominat).unwrap().try_into().unwrap();
        }
    }

    fn rest_mul(&mut self, mut n2: i128, mut d2: i128) {
        let (mut n1, mut d1): (i128, i128) = (self.numerat.into(), self.denominat.get().into());
        // we reduce the fractions as much as possible before-hand,
        // to avoid many cases of overflows...
        Self::normalize_inner(&mut n1, &mut d2);
        Self::normalize_inner(&mut n2, &mut d1);
        n1 *= n2;
        d1 *= d2;
        Self::normalize_inner(&mut n1, &mut d1);
        self.numerat = n1.try_into().unwrap();
        self.denominat = u64::try_from(d1).unwrap().try_into().unwrap();
    }
}

impl ops::MulAssign<i64> for Fraction {
    fn mul_assign(&mut self, rhs: i64) {
        use std::cmp::Ordering;
        let rhs = match rhs.cmp(&0) {
            Ordering::Equal => {
                *self = Fraction::default();
                return;
            }
            Ordering::Less => {
                self.numerat *= -1;
                -rhs
            }
            Ordering::Greater => rhs,
        };
        assert!(rhs > 0);
        let mut rhs: u64 = rhs.try_into().unwrap();
        for (&k, v) in &mut self.primfacts {
            let k: u64 = k.into();
            while (rhs % k) == 0u64 {
                rhs /= k;
                *v += 1;
            }
        }
        self.numerat *= i64::try_from(rhs).unwrap();
        self.normalize();
    }
}

impl ops::DivAssign<i64> for Fraction {
    fn div_assign(&mut self, mut rhs: i64) {
        assert_ne!(rhs, 0);
        if rhs < 0 {
            self.numerat *= -1;
            rhs *= -1;
        }
        assert!(rhs > 0);
        let mut rhs: u64 = rhs.try_into().unwrap();
        for (&k, v) in &mut self.primfacts {
            let k: u64 = k.into();
            while (rhs % k) == 0u64 {
                rhs /= k;
                *v -= 1;
            }
        }
        self.denominat = (rhs * self.denominat.get()).try_into().unwrap();
        self.normalize();
    }
}

impl<'a> ops::MulAssign<&'a Fraction> for Fraction {
    fn mul_assign(&mut self, rhs: &'a Fraction) {
        for ((&k1, v1), (&k2, v2)) in self.primfacts.iter_mut().zip(rhs.primfacts.iter()) {
            assert_eq!(k1, k2);
            *v1 += v2;
        }
        self.rest_mul(rhs.numerat.into(), rhs.denominat.get().into());
    }
}

impl<'a> ops::DivAssign<&'a Fraction> for Fraction {
    fn div_assign(&mut self, rhs: &'a Fraction) {
        for ((&k1, v1), (&k2, v2)) in self.primfacts.iter_mut().zip(rhs.primfacts.iter()) {
            assert_eq!(k1, k2);
            *v1 -= v2;
        }
        self.rest_mul(rhs.denominat.get().into(), rhs.numerat.into());
    }
}

impl<'a> ops::Add<&'a Fraction> for &'a Fraction {
    type Output = Fraction;
    fn add(self, rhs: &'a Fraction) -> Fraction {
        let (a, b) = (self, rhs);
        let rft = euclid(a.denominat.get(), b.denominat.get());
        let min_prims = a
            .primfacts
            .iter()
            .zip(b.primfacts.iter())
            .map(|((&k1, &v1), (&k2, &v2))| {
                assert_eq!(k1, k2);
                let min = std::cmp::min(v1, v2);
                (
                    k1,
                    (
                        min,
                        u32::try_from(v1 - min).unwrap(),
                        u32::try_from(v2 - min).unwrap(),
                    ),
                )
            })
            .collect::<BTreeMap<u8, (i32, u32, u32)>>();
        let mut nnm_a: i64 = a.numerat;
        let mut nnm_b: i64 = b.numerat;
        nnm_a *= i64::try_from(b.denominat.get() / rft).unwrap();
        nnm_b *= i64::try_from(a.denominat.get() / rft).unwrap();
        nnm_a *= i64::try_from(
            min_prims
                .iter()
                .map(|(&k, &(_, v, _))| u64::from(k).checked_pow(v).unwrap())
                .product::<u64>(),
        )
        .unwrap();
        nnm_b *= i64::try_from(
            min_prims
                .iter()
                .map(|(&k, &(_, _, v))| u64::from(k).checked_pow(v).unwrap())
                .product::<u64>(),
        )
        .unwrap();
        let mut ret = Fraction {
            primfacts: min_prims.iter().map(|(&k, &(v, _, _))| (k, v)).collect(),
            numerat: nnm_a + nnm_b,
            denominat: ((a.denominat.get() * b.denominat.get()) / rft)
                .try_into()
                .unwrap(),
        };
        ret.normalize();
        ret
    }
}
